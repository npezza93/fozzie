#![feature(test)]
#[macro_use]
extern crate clap;
extern crate terminal_size;
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

use choice::Choice;
use choices::Choices;
use config::Config;
use search::Search;
use std::error::Error;
use std::io::{self, Read};
use terminal::Terminal;
use termion::event::Key;
use rayon::prelude::*;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();
        let mut exit_code = 0;

        let mut terminal = Terminal::new()?;
        let stdin = io::stdin();
        let mut stdin_lock = stdin.lock();

        let mut buffer = String::new();

       if termion::is_tty(&stdin_lock) {
           clap::Error::with_description(
               "No input given", clap::ErrorKind::MissingRequiredArgument
           ).exit();
        }

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
                    Key::Alt(c) => handle_alt_key(c, &mut search, &mut choices, |text| {
                        terminal.print(text);
                    }),
                    Key::Char('\n') => {
                        choices.select(&mut terminal);
                        break;
                    }
                    Key::Char('\t') => {
                        terminal.print(&search.set_query(choices.current_match().searchable));
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

// Word movement only redraws the query. Editing it must also refresh the matches.
fn handle_alt_key<F: FnMut(&str)>(
    key: char,
    search: &mut Search,
    choices: &mut Choices,
    mut print: F,
) {
    match key as u8 {
        b'b' => print(&search.left_word()),
        b'f' => print(&search.right_word()),
        127 => {
            print(&search.backspace_word());
            print(&choices.filter(&search.query));
        }
        100 => {
            print(&search.delete_word());
            print(&choices.filter(&search.query));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor;

    fn config() -> Config {
        Config {
            lines: 1, prompt: "> ".to_string(), show_scores: false,
            query: None, delimiter: None, field: None, output: None,
            benchmark: false,
        }
    }

    #[test]
    fn word_movement_preserves_query_selection_and_results() {
        let config = config();
        let input = vec![Choice::new("foo bar", &config), Choice::new("foo bars", &config)];
        let mut choices = Choices::new(1, &input, false);
        let mut search = Search::new("> ");
        search.set_query("foo bar");
        choices.filter(&search.query);
        choices.next();
        assert_eq!("foo bars", choices.current_match().searchable);

        // Include repeated movement at both boundaries and a scrolled selection.
        for (key, column) in [('b', 7), ('b', 3), ('b', 3), ('f', 7), ('f', 10), ('f', 10)] {
            let mut output = Vec::new();
            handle_alt_key(key, &mut search, &mut choices, |text| output.push(text.to_string()));
            assert_eq!("foo bar".chars().collect::<Vec<_>>(), search.query);
            assert_eq!("foo bars", choices.current_match().searchable);
            assert_eq!(vec![format!("{}\r> foo bar{}", cursor::clear_line(), cursor::col(column))], output);
        }
        choices.next();
        assert_eq!("foo bar", choices.current_match().searchable);
        choices.previous();
        assert_eq!("foo bars", choices.current_match().searchable);
    }

    #[test]
    fn word_deletion_still_refreshes_matches() {
        let config = config();
        let input = vec![Choice::new("foo bar", &config), Choice::new("foo bars", &config)];
        for key in ['\u{7f}', 'd'] {
            let mut choices = Choices::new(1, &input, false);
            let mut search = Search::new("> ");
            search.set_query("foo bar");
            choices.filter(&search.query);
            choices.next();
            if key == 'd' {
                search.left_word();
            }
            let mut output = Vec::new();
            handle_alt_key(key, &mut search, &mut choices, |text| output.push(text.to_string()));
            assert_eq!("foo ".chars().collect::<Vec<_>>(), search.query);
            assert_eq!("foo bar", choices.current_match().searchable);
            assert_eq!(2, output.len());
            assert!(output[1].contains(cursor::clear_screen_down()));
        }
    }
}
