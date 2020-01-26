#[macro_use]
extern crate clap;

use choices::Choices;
use config::Config;
use search::Search;
use std::error::Error;
use std::io::{stdin, stdout};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub mod choice;
pub mod choices;
pub mod color;
pub mod config;
pub mod cursor;
pub mod search;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let mut search = Search::new(config.prompt, stdout_output());
    let mut choices = Choices::new(config.lines, stdout_output(), stdin().lock());

    choices.inital_draw();
    search.render();

    let tty = termion::get_tty()?;
    for c in tty.keys() {
        match c.unwrap() {
            Key::Char(c) => search.keypress(c),
            Key::Ctrl('u') => search.clear(),
            Key::Ctrl('c') => break,
            Key::Esc => break,
            Key::Left => search.left(),
            Key::Right => search.right(),
            Key::Up => choices.previous(),
            Key::Down => choices.next(),
            Key::Backspace => search.backspace(),
            _ => {}
        }
    }

    Ok(())
}

fn stdout_output() -> termion::raw::RawTerminal<std::io::Stdout> {
    stdout().into_raw_mode().unwrap()
}
