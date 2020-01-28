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
        self.output.write(text.as_bytes()).unwrap();
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
    use std::io::Cursor;

    #[test]
    fn test_render() {
        let mut cursor = Cursor::new(vec![]);
        let mut search = Search::new("> ".to_string(), &mut cursor);
        search.render();

        let actual = cursor.into_inner();

        assert_eq!(
            format!("{}\r> {}", cursor::clear_line(), cursor::col(3)),
            String::from_utf8(actual).unwrap()
        );
    }

    #[test]
    fn test_keypress() {
        let mut cursor = Cursor::new(vec![]);
        let mut search = Search::new("> ".to_string(), &mut cursor);
        search.keypress('b');

        let actual = cursor.into_inner();

        assert_eq!(
            format!("{}\r> b{}", cursor::clear_line(), cursor::col(4)),
            String::from_utf8(actual).unwrap()
        );
    }

    #[test]
    fn test_backspace() {
        let mut cursor = Cursor::new(vec![]);
        let mut search = Search::new("> ".to_string(), &mut cursor);
        search.query = vec!['a', 'b', 'c'];
        search.position = 3;
        search.backspace();

        let actual = cursor.into_inner();

        assert_eq!(
            format!("{}\r> ab{}", cursor::clear_line(), cursor::col(5)),
            String::from_utf8(actual).unwrap()
        );
    }

    #[test]
    fn test_left() {
        let mut cursor = Cursor::new(vec![]);
        let mut search = Search::new("> ".to_string(), &mut cursor);
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;
        search.render();
        search.left();

        let actual = cursor.into_inner();

        assert_eq!(
            format!(
                "{}\r> abc{}{}",
                cursor::clear_line(),
                cursor::col(4),
                cursor::left(1)
            ),
            String::from_utf8(actual).unwrap()
        );
    }

    #[test]
    fn test_right() {
        let mut cursor = Cursor::new(vec![]);
        let mut search = Search::new("> ".to_string(), &mut cursor);
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;
        search.render();
        search.right();

        let actual = cursor.into_inner();

        assert_eq!(
            format!(
                "{}\r> abc{}{}",
                cursor::clear_line(),
                cursor::col(4),
                cursor::right(1)
            ),
            String::from_utf8(actual).unwrap()
        );
    }

    #[test]
    fn test_clear() {
        let mut cursor = Cursor::new(vec![]);
        let mut search = Search::new("> ".to_string(), &mut cursor);
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;
        search.clear();

        let actual = cursor.into_inner();

        assert_eq!(
            format!("{}\r> {}", cursor::clear_line(), cursor::col(3)),
            String::from_utf8(actual).unwrap()
        );
    }
}
