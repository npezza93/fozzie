#[macro_use]
extern crate clap;

use choices::Choices;
use config::Config;
use search::Search;
use std::error::Error;
use std::io::stdout;
use std::process;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub mod choices;
pub mod config;
pub mod cursor;
pub mod search;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let mut search = Search::new(config.prompt, stdout_output());
    let mut choices = Choices::new(config.lines, stdout_output());

    choices.render();
    search.render();

    let tty = termion::get_tty()?;
    for c in tty.keys() {
        match c.unwrap() {
            Key::Char(c) => search.keypress(c),
            Key::Ctrl('u') => search.clear(),
            Key::Ctrl('c') => process::exit(1),
            Key::Esc => process::exit(1),
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

fn stdout_output() -> termion::raw::RawTerminal<std::io::Stdout> {
    stdout().into_raw_mode().unwrap()
}
