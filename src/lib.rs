#![feature(test)]
#[macro_use]
extern crate clap;
extern crate term_size;
extern crate test;

pub mod bonus;
pub mod choices;
pub mod choice;
pub mod color;
pub mod config;
pub mod cursor;
pub mod matcher;
pub mod matrix;
pub mod scorer;
pub mod search;
pub mod terminal;

use rayon::prelude::*;
use choice::Choice;
use choices::Choices;
use config::Config;
use search::Search;
use std::error::Error;
use std::io::{self, Read};
use terminal::Terminal;
use termion::event::Key;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();
        let mut exit_code = 0;

        let mut terminal = Terminal::new()?;
        let stdin = io::stdin();
        let mut stdin_lock = stdin.lock();

        let mut buffer = String::new();

        stdin_lock.read_to_string(&mut buffer)?;
        let parsed_choices: Vec<Choice> =
            buffer.
            par_lines().
            map(|choice| Choice::new(choice, &config)).
            collect();

        let mut search = Search::new(config.prompt);
        let mut choices = Choices::new(config.lines, &parsed_choices, config.show_scores);

        if config.benchmark {
            let query = config.query.unwrap().chars().collect::<Vec<char>>();
            for _ in 0..100 {
                choices.filter(&query);
            }
        } else {
            choices.initial_draw(&mut terminal);
            match config.query {
                Some(query) => {
                    terminal.print(&search.set_query(&query));
                    terminal.print(&choices.filter(&search.query));
                }
                None => terminal.print(&search.draw())
            }

            for c in terminal.keys()? {
                match c.unwrap() {
                    Key::Alt(c) => match c as u8 {
                        b'b' => {
                            terminal.print(&search.left_word());
                            terminal.print(&choices.filter(&search.query));
                        },
                        b'f' => {
                            terminal.print(&search.right_word());
                            terminal.print(&choices.filter(&search.query));
                        },
                        127 => {
                            terminal.print(&search.backspace_word());
                            terminal.print(&choices.filter(&search.query));
                        },
                        100 => {
                            terminal.print(&search.delete_word());
                            terminal.print(&choices.filter(&search.query));
                        },
                        _ => {}
                    },
                    Key::Char('\n') => {
                        choices.select(&mut terminal);
                        break;
                    }
                    Key::Char('\t') => {
                        terminal.print(&search.set_query(&choices.current_match().searchable));
                        terminal.print(&choices.filter(&search.query));
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
        }

        Ok(exit_code)
    }
}
