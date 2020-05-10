#[macro_use]
extern crate clap;

use clap::Shell;
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

    app.gen_completions("fozzie", Shell::Bash, &outdir);
    app.gen_completions("fozzie", Shell::Fish, &outdir);
    app.gen_completions("fozzie", Shell::Zsh, outdir);
}
