#[macro_use]
extern crate clap;

pub mod choice;
pub mod choices;
pub mod color;
pub mod config;
pub mod cursor;
pub mod matcher;
pub mod search;
pub mod terminal;

use choices::Choices;
use config::Config;
use search::Search;
use std::error::Error;
use std::io::{stdin, BufRead};
use terminal::Terminal;
use termion::event::Key;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();
        let mut exit_code = 0;

        let mut terminal = Terminal::new()?;
        let parsed_choices: Vec<String> = stdin().lock().lines().map(Result::unwrap).collect();
        let mut search = Search::new(config.prompt);
        let mut choices = Choices::new(config.lines, &parsed_choices);

        choices.initial_draw(&mut terminal);
        terminal.print(&search.draw());

        for c in terminal.keys()? {
            match c.unwrap() {
                Key::Alt(c) => match c as u8 {
                    b'b' => terminal.print(&search.left_word()),
                    b'f' => terminal.print(&search.right_word()),
                    127 => terminal.print(&search.backspace_word()),
                    100 => terminal.print(&search.delete_word()),
                    _ => {}
                },
                Key::Char('\n') => {
                    choices.select(&mut terminal);
                    break;
                }
                Key::Char(c) => {
                    terminal.print(&search.keypress(c));
                    terminal.print(&choices.filter(&search.query));
                }
                Key::Ctrl('u') => terminal.print(&search.clear()),
                Key::Up => terminal.print(&choices.previous()),
                Key::Down => terminal.print(&choices.next()),
                Key::Esc | Key::Ctrl('c') => {
                    exit_code = 1;
                    terminal.print(&choices.cancel());
                    break;
                }
                Key::Left => {
                    if let Some(text) = search.left() {
                        terminal.print(text);
                    }
                }
                Key::Right => {
                    if let Some(text) = search.right() {
                        terminal.print(text);
                    }
                }
                Key::Backspace => {
                    if let Some(text) = search.backspace() {
                        terminal.print(&text);
                        terminal.print(&choices.filter(&search.query));
                    }
                }
                Key::Ctrl('d') => {
                    if let Some(text) = search.delete() {
                        terminal.print(&text);
                        terminal.print(&choices.filter(&search.query));
                    }
                }
                _ => {}
            }
        }

        Ok(exit_code)
    }
}
