use crate::cursor;

pub struct Search {
    pub query: Vec<char>,
    position: usize,
    prompt: String,
}

impl Search {
    pub fn new<S: Into<String>>(prompt: S) -> Search {
        Search {
            query: vec![],
            position: 0,
            prompt: prompt.into(),
        }
    }

    pub fn draw(&self) -> String {
        let query: String = self.query.iter().collect();
        let current_col = self.prompt.chars().count() + self.position + 1;

        format!(
            "{}\r{}{}{}",
            cursor::clear_line(),
            self.prompt,
            query,
            cursor::col(current_col)
        )
    }

    pub fn keypress(&mut self, character: char) -> String {
        self.query.insert(self.position, character);
        self.position += 1;

        self.draw()
    }

    pub fn backspace(&mut self) -> Option<String> {
        if self.position > 0 {
            self.position -= 1;
            self.query.remove(self.position);
            Some(self.draw())
        } else {
            None
        }
    }

    pub fn left(&mut self) -> Option<&str> {
        if self.position > 0 {
            self.position -= 1;
            Some(cursor::left())
        } else {
            None
        }
    }

    pub fn right(&mut self) -> Option<&str> {
        if self.query.len() != self.position {
            self.position += 1;
            Some(cursor::right())
        } else {
            None
        }
    }

    pub fn clear(&mut self) -> String {
        self.query = vec![];
        self.position = 0;
        self.draw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw() {
        let search = Search::new("> ");

        assert_eq!(
            format!("{}\r> {}", cursor::clear_line(), cursor::col(3)),
            search.draw()
        );
    }

    #[test]
    fn test_keypress() {
        let mut search = Search::new("> ");

        assert_eq!(
            format!("{}\r> b{}", cursor::clear_line(), cursor::col(4)),
            search.keypress('b')
        );
        assert_eq!(1, search.position);
    }

    #[test]
    fn test_backspace() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 3;

        assert_eq!(
            format!("{}\r> ab{}", cursor::clear_line(), cursor::col(5)),
            search.backspace().unwrap()
        );
        assert_eq!(2, search.position);
    }

    #[test]
    fn test_backspace_none() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 0;

        assert!(search.backspace().is_none());
        assert_eq!(0, search.position);
    }

    #[test]
    fn test_left() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;

        assert_eq!(format!("{}", cursor::left()), search.left().unwrap());
        assert_eq!(0, search.position);
    }

    #[test]
    fn test_left_none() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 0;

        assert!(search.left().is_none());
        assert_eq!(0, search.position);
    }

    #[test]
    fn test_right() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;

        assert_eq!(format!("{}", cursor::right()), search.right().unwrap());
        assert_eq!(2, search.position);
    }

    #[test]
    fn test_right_none() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 3;

        assert!(search.right().is_none());
        assert_eq!(3, search.position);
    }

    #[test]
    fn test_clear() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;
        search.clear();

        assert_eq!(
            format!("{}\r> {}", cursor::clear_line(), cursor::col(3)),
            search.clear()
        );
        assert_eq!(0, search.position);
    }
}
