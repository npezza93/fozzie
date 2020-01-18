#[macro_use]
extern crate clap;

use choices::Choices;
use config::Config;
use search::Search;
use std::error::Error;
use termion::event::Key;
use termion::input::TermRead;

use std::io::stdout;
use termion::raw::IntoRawMode;

pub mod choices;
pub mod config;
pub mod cursor;
pub mod search;
pub mod writer;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let mut search = Search::new(config.prompt);
    let choices = Choices::new(config.lines);

    choices.render();
    search.render();

    let _stdout = stdout().into_raw_mode().unwrap(); // activate raw mode
    let tty = termion::get_tty()?;
    for c in tty.keys() {
        match c.unwrap() {
            Key::Char(c) => search.keypress(c),
            Key::Alt(c) => println!("^{}", c),
            Key::Ctrl(c) => println!("*{}", c),
            Key::Esc => break,
            Key::Left => search.left(),
            Key::Right => search.right(),
            Key::Up => println!("↑"),
            Key::Down => println!("↓"),
            Key::Backspace => search.backspace(),
            _ => {}
        }
    }

    Ok(())
}
