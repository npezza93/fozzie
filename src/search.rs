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

    pub fn clear(&mut self) {
        self.query = vec![];
        self.position = 0;
        self.render();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_render() {
        let output = NamedTempFile::new().expect("err");
        let mut search = Search::new("> ".to_string(), &output);
        search.render();

        let mut output = output.reopen().expect("err");
        let mut actual = String::new();
        output.read_to_string(&mut actual).expect("Err");

        assert_eq!(
            format!("{}\r> {}", cursor::clear_line(), cursor::col(3)),
            actual
        );
    }

    #[test]
    fn test_keypress() {
        let output = NamedTempFile::new().expect("err");
        let mut search = Search::new("> ".to_string(), &output);
        search.keypress('b');

        let mut output = output.reopen().expect("err");
        let mut actual = String::new();
        output.read_to_string(&mut actual).expect("Err");

        assert_eq!(
            format!("{}\r> b{}", cursor::clear_line(), cursor::col(4)),
            actual
        );
    }

    #[test]
    fn test_backspace() {
        let output = NamedTempFile::new().expect("err");
        let mut search = Search::new("> ".to_string(), &output);
        search.query = vec!['a', 'b', 'c'];
        search.position = 3;
        search.backspace();

        let mut output = output.reopen().expect("err");
        let mut actual = String::new();
        output.read_to_string(&mut actual).expect("Err");

        assert_eq!(
            format!("{}\r> ab{}", cursor::clear_line(), cursor::col(5)),
            actual
        );
    }

    #[test]
    fn test_left() {
        let output = NamedTempFile::new().expect("err");
        let mut search = Search::new("> ".to_string(), &output);
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;
        search.render();
        search.left();

        let mut output = output.reopen().expect("err");
        let mut actual = String::new();
        output.read_to_string(&mut actual).expect("Err");

        assert_eq!(
            format!(
                "{}\r> abc{}{}",
                cursor::clear_line(),
                cursor::col(4),
                cursor::left(1)
            ),
            actual
        );
    }

    #[test]
    fn test_right() {
        let output = NamedTempFile::new().expect("err");
        let mut search = Search::new("> ".to_string(), &output);
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;
        search.render();
        search.right();

        let mut output = output.reopen().expect("err");
        let mut actual = String::new();
        output.read_to_string(&mut actual).expect("Err");

        assert_eq!(
            format!(
                "{}\r> abc{}{}",
                cursor::clear_line(),
                cursor::col(4),
                cursor::right(1)
            ),
            actual
        );
    }

    #[test]
    fn test_clear() {
        let output = NamedTempFile::new().expect("err");
        let mut search = Search::new("> ".to_string(), &output);
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;
        search.clear();

        let mut output = output.reopen().expect("err");
        let mut actual = String::new();
        output.read_to_string(&mut actual).expect("Err");

        assert_eq!(
            format!("{}\r> {}", cursor::clear_line(), cursor::col(3)),
            actual
        );
    }
}
