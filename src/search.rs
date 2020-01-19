use crate::cursor;
use std::io::Write;

pub struct Search<W> {
    query: Vec<char>,
    position: usize,
    prompt: String,
    output: W,
}

impl<W: Write> Search<W> {
    pub fn new(prompt: String, output: W) -> Search<W> {
        Search {
            query: vec![],
            position: 0,
            prompt,
            output,
        }
    }

    fn print(&mut self, text: String) {
        write!(self.output, "{}", text).unwrap();
        self.output.flush().unwrap();
    }

    pub fn render(&mut self) {
        let query: String = self.query.iter().collect();
        let current_col = self.prompt.chars().count() + self.position + 1;

        self.print(format!(
            "{}\r{}{}{}",
            cursor::clear_line(),
            self.prompt,
            query,
            cursor::col(current_col)
        ));
    }

    pub fn keypress(&mut self, character: char) {
        self.query.insert(self.position, character);
        self.position += 1;

        self.render();
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
            self.print(cursor::left(1));
        }
    }

    pub fn right(&mut self) {
        if self.query.len() != self.position {
            self.position += 1;
            self.print(cursor::right(1));
        }
    }
}
