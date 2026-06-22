// echo: read all of stdin and write it back unchanged to stdout.
// Entry contract: argv/stdin in, stdout/stderr out (WASI command).
use std::io::{self, Read, Write};

fn main() {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() {
        std::process::exit(1);
    }
    if io::stdout().write_all(&input).is_err() {
        std::process::exit(1);
    }
}
