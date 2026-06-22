// vibration-fft: edge preprocessing for predictive maintenance.
// Reads a raw accelerometer waveform (comma/whitespace separated floats) from stdin,
// runs an FFT + time-domain stats on-site, and emits a tiny JSON verdict to stdout.
// This is the "ingress(large raw) -> runtime(reduce) -> egress(small result)" pattern:
// ~32KB waveform in, ~60B verdict out. Entry contract: argv/stdin in, stdout/stderr out.
//
// Parameters (fs = sample rate Hz, channels = analysis windows/axes per call) can be given
// two ways so every transport works:
//   - argv:        `<fs> <channels>`  (HTTP sync/async carry argv)
//   - stdin header: an optional first line containing `=` tokens, e.g. `fs=1000 channels=256`
//     followed by the waveform. MQTT carries only the payload as stdin (no argv), so the
//     header is how MQTT invocations dial the load. Leading `#` is allowed.
// Precedence: defaults < stdin header < argv (argv wins when present).
// channels scales CPU as channels * O(n log n): each channel analyses a phase-shifted view
// of the capture (overlapping window); the worst (most peaky) channel drives the verdict.
use std::io::{self, Read, Write};

// parse_header reads `fs=`/`channels=` (alias `ch=`) tokens from a header line into the refs.
fn parse_header(line: &str, fs: &mut f64, channels: &mut usize) {
    for tok in line
        .trim_start_matches('#')
        .split(|c: char| c.is_whitespace() || c == ',')
    {
        if let Some((k, v)) = tok.split_once('=') {
            match k.trim() {
                "fs" => {
                    if let Ok(x) = v.trim().parse() {
                        *fs = x;
                    }
                }
                "channels" | "ch" => {
                    if let Ok(x) = v.trim().parse() {
                        *channels = x;
                    }
                }
                _ => {}
            }
        }
    }
}

// analyze runs one FFT pass over a phase-shifted view of samples and returns
// (rms, peak, dom_hz, peakiness, anomaly) for that window.
fn analyze(samples: &[f64], shift: usize, fs: f64) -> (f64, f64, f64, f64, bool) {
    let n = samples.len();
    let mut re = vec![0.0f64; n];
    let mut im = vec![0.0f64; n];
    let mut sumsq = 0.0f64;
    let mut peak = 0.0f64;
    for i in 0..n {
        let x = samples[(i + shift) % n];
        re[i] = x;
        sumsq += x * x;
        peak = peak.max(x.abs());
    }
    let rms = (sumsq / n as f64).sqrt();
    fft(&mut re, &mut im);
    let half = n / 2;
    let mut dom_bin = 0usize;
    let mut dom_mag = 0.0f64;
    let mut sum_mag = 0.0f64;
    for k in 1..half {
        let m = (re[k] * re[k] + im[k] * im[k]).sqrt();
        sum_mag += m;
        if m > dom_mag {
            dom_mag = m;
            dom_bin = k;
        }
    }
    let bins = half.saturating_sub(1).max(1) as f64;
    let mean_mag = sum_mag / bins;
    let peakiness = if mean_mag > 0.0 { dom_mag / mean_mag } else { 0.0 };
    let dom_hz = dom_bin as f64 * fs / n as f64;
    (rms, peak, dom_hz, peakiness, peakiness > 8.0)
}

// fft runs an in-place iterative radix-2 Cooley-Tukey FFT (no external deps).
fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    // bit-reversal permutation
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    // butterfly stages
    let mut len = 2usize;
    while len <= n {
        let ang = -2.0 * std::f64::consts::PI / (len as f64);
        let (wr, wi) = (ang.cos(), ang.sin());
        let mut i = 0usize;
        while i < n {
            let (mut cr, mut ci) = (1.0f64, 0.0f64);
            for k in 0..len / 2 {
                let a = i + k;
                let b = a + len / 2;
                let tr = cr * re[b] - ci * im[b];
                let ti = cr * im[b] + ci * re[b];
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let ncr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = ncr;
            }
            i += len;
        }
        len <<= 1;
    }
}

fn main() {
    let mut fs: f64 = 1000.0;
    let mut channels: usize = 1;

    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }

    // optional stdin header (first line with '=' tokens) — the only param path for MQTT.
    let body: &str = match input.split_once('\n') {
        Some((first, rest)) if first.contains('=') => {
            parse_header(first, &mut fs, &mut channels);
            rest
        }
        _ => input.as_str(),
    };

    // argv overrides the header when present (HTTP carries argv; MQTT does not).
    let mut args = std::env::args().skip(1);
    if let Some(x) = args.next().and_then(|s| s.parse().ok()) {
        fs = x;
    }
    if let Some(x) = args.next().and_then(|s| s.parse().ok()) {
        channels = x;
    }
    channels = channels.clamp(1, 1_000_000);

    let mut samples: Vec<f64> = body
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();

    if samples.len() < 2 {
        let _ = writeln!(io::stderr(), "need >=2 numeric samples");
        std::process::exit(1);
    }

    // radix-2 FFT needs a power-of-two length: use the largest power-of-two prefix.
    let n = 1usize << (usize::BITS - 1 - (samples.len()).leading_zeros());
    samples.truncate(n);

    // process `channels` phase-shifted windows; keep the worst (most peaky) verdict and
    // count how many windows tripped the anomaly threshold.
    let mut worst = (0.0f64, 0.0f64, 0.0f64, 0.0f64, false);
    let mut anomaly_channels = 0usize;
    for c in 0..channels {
        let shift = (c * 97) % n;
        let r = analyze(&samples, shift, fs);
        if r.4 {
            anomaly_channels += 1;
        }
        if r.3 > worst.3 {
            worst = r;
        }
    }
    let (rms, peak, dom_hz, peakiness, anomaly) = worst;

    let out = format!(
        "{{\"n\":{},\"channels\":{},\"rms\":{:.4},\"peak\":{:.4},\"dom_hz\":{:.2},\"peakiness\":{:.2},\"anomaly\":{},\"anomaly_channels\":{}}}\n",
        n, channels, rms, peak, dom_hz, peakiness, anomaly, anomaly_channels
    );
    if io::stdout().write_all(out.as_bytes()).is_err() {
        std::process::exit(1);
    }
}
