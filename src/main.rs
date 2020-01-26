use fozzie::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    std::process::exit(App::run()?);
}
