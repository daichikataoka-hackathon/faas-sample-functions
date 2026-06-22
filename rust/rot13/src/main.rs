// rot13: classic Caesar ROT13 over ASCII letters; other bytes pass through.
// Entry contract: argv/stdin in, stdout/stderr out (WASI command).
use std::io::{self, Read, Write};

fn rot13(b: u8) -> u8 {
    match b {
        b'a'..=b'z' => b'a' + (b - b'a' + 13) % 26,
        b'A'..=b'Z' => b'A' + (b - b'A' + 13) % 26,
        _ => b,
    }
}

fn main() {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() {
        std::process::exit(1);
    }
    let out: Vec<u8> = input.iter().map(|&b| rot13(b)).collect();
    if io::stdout().write_all(&out).is_err() {
        std::process::exit(1);
    }
}
