#[macro_use]
extern crate clap;

use choices::Choices;
use config::Config;
use raw_tty::IntoRawMode;
use search::Search;
use std::error::Error;
use std::fs;
use std::io::stdin;
use termion::event::Key;
use termion::input::TermRead;

pub mod choice;
pub mod choices;
pub mod color;
pub mod config;
pub mod cursor;
pub mod search;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?;

    let mut search = Search::new(config.prompt, tty.try_clone()?);
    let mut choices = Choices::new(config.lines, tty.try_clone()?, stdin().lock());

    choices.inital_draw();
    search.render();

    for c in tty.into_raw_mode()?.keys() {
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
