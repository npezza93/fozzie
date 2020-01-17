use std::io::Write;
use crate::settings;
use crate::cursor;

pub struct Search {
    query: Vec<char>,
    position: usize,
}

impl Search {
    pub fn new() -> Search {
        Search { query: vec![], position: 0 }
    }

    fn current_col(&self, settings: &settings::Settings) -> usize {
        settings.prompt.chars().count() + self.position + 1
    }

    pub fn render(&self, settings: &settings::Settings) {
        let query: String = self.query.iter().collect();
        let mut stdout = settings.stdout();

        cursor::clear_line(&settings);
        write!(stdout, "\r{}{}", settings.prompt, query).unwrap();
        cursor::col(self.current_col(&settings), &settings);
        stdout.flush().unwrap();
    }

    pub fn keypress(&mut self, character: char, settings: &settings::Settings) {
        self.query.push(character);
        self.position += 1;
        write!(settings.stdout(), "{}", character).unwrap();
    }

    pub fn backspace(&mut self, settings: &settings::Settings) {
        if self.position > 0 {
            self.position -= 1;
            self.query.remove(self.position);
            self.render(&settings);
        }
    }

    pub fn left(&mut self, settings: &settings::Settings) {
        if self.position > 0 {
            self.position -= 1;
            cursor::left(1, &settings);
        }
    }

    pub fn right(&mut self, settings: &settings::Settings) {
        if self.query.len() != self.position {
            self.position += 1;
            cursor::right(1, &settings);
        }
    }
}
