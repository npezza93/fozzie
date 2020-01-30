#[macro_use]
extern crate clap;

pub mod choice;
pub mod choices;
pub mod color;
pub mod config;
pub mod cursor;
pub mod matcher;
pub mod search;

use choices::Choices;
use config::Config;
use raw_tty::IntoRawMode;
use search::Search;
use std::error::Error;
use std::fs::{self, File};
use termion::event::Key;
use termion::input::TermRead;
use std::io::{self, stdin, BufRead, Write};

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();
        let mut exit_code = 0;

        let mut tty = tty()?;
        let parsed_choices: Vec<String> = stdin().lock().lines().map(Result::unwrap).collect();
        let mut search = Search::new(config.prompt);
        let mut choices = Choices::new(config.lines, &parsed_choices);

        write(&mut tty, &search.draw());
        write(&mut tty, &choices.draw());

        for c in tty.try_clone()?.into_raw_mode()?.keys() {
            match c.unwrap() {
                Key::Char('\n') => {
                    write(&mut tty, &choices.select());
                    break;
                },
                Key::Char('u') => write(&mut tty, &search.clear()),
                Key::Char(c) => write(&mut tty, &search.keypress(c)),
                Key::Esc | Key::Ctrl('c') => {
                    exit_code = 1;
                    write(&mut tty, &choices.cancel());
                    break;
                },
                Key::Left => {
                    if let Some(text) = search.left() {
                        write(&mut tty, text);
                   }
                },
                Key::Right => {
                    if let Some(text) = search.right() {
                        write(&mut tty, text);
                   }
                },
                Key::Up => write(&mut tty, &choices.previous()),
                Key::Down => {
                    write(&mut tty, &choices.next());
                },
                Key::Backspace => {
                    if let Some(text) = search.backspace() {
                        write(&mut tty, &text);
                   }
                },
                _ => {}
            }
        }

        Ok(exit_code)
    }
}

fn tty() -> Result<File, io::Error> {
    fs::OpenOptions::new().read(true).write(true).open("/dev/tty")
}

fn write(output: &mut File, text: &str) {
    output.write(text.as_bytes()).unwrap();
    output.flush().unwrap();
}
