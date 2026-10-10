use clap_complete::{generate_to, Shell};
use std::env;
use std::path::PathBuf;

include!("src/config.rs");

fn main() {
    let mut app = Config::menu();
    let mut outdir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR variable was not set"));

    // This will output the completions in the target/release or target/debug directories.
    outdir.pop();
    outdir.pop();
    outdir.pop();

    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
        generate_to(shell, &mut app, "fozzie", &outdir)
            .expect("Could not generate shell completions");
    }
}
