use crate::cursor;
use crate::writer;

pub struct Search {
    query: Vec<char>,
    position: usize,
    prompt: String,
}

impl Search {
    pub fn new(prompt: String) -> Search {
        Search {
            query: vec![],
            position: 0,
            prompt,
        }
    }

    pub fn render(&self) {
        let query: String = self.query.iter().collect();
        let current_col = self.prompt.chars().count() + self.position + 1;

        writer::print(format!(
            "{}\r{}{}{}",
            cursor::clear_line(),
            self.prompt,
            query,
            cursor::col(current_col)
        ));
    }

    pub fn keypress(&mut self, character: char) {
        self.query.push(character);
        self.position += 1;

        writer::print(character.to_string());
    }

    pub fn backspace(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            self.query.remove(self.position);
            self.render();
        }
    }

    pub fn left(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            writer::print(cursor::left(1));
        }
    }

    pub fn right(&mut self) {
        if self.query.len() != self.position {
            self.position += 1;
            writer::print(cursor::right(1));
        }
    }
}
