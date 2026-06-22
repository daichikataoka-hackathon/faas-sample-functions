// uppercase: read all of stdin (UTF-8), uppercase it, write to stdout.
// Entry contract: argv/stdin in, stdout/stderr out (WASI command).
use std::io::{self, Read, Write};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }
    if io::stdout().write_all(input.to_uppercase().as_bytes()).is_err() {
        std::process::exit(1);
    }
}
