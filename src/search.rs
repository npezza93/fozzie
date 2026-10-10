use crate::{cursor, terminal};
use regex::Regex;
use std::sync::OnceLock;

fn word_start_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\b\w").unwrap())
}

fn word_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\w\b").unwrap())
}

pub struct Search {
    pub query: Vec<char>,
    position: usize,
    prompt: String,
    // Retain the borrowed movement-result API while returning a full redraw.
    redraw: String,
}

impl Search {
    pub fn new<S: Into<String>>(prompt: S) -> Search {
        Search {
            query: vec![],
            position: 0,
            prompt: prompt.into(),
            redraw: String::new(),
        }
    }

    pub fn draw(&self) -> String {
        self.draw_with_width(terminal::columns())
    }

    fn draw_with_width(&self, columns: Option<usize>) -> String {
        let query: String = self.query.iter().collect();
        let Some(columns) = columns else {
            return format!("{}\r{}{}{}", cursor::clear_line(), self.prompt,
                query, cursor::col(terminal::cell_width(&self.prompt)
                    + terminal::cell_width(&self.query[..self.position].iter().collect::<String>()) + 1));
        };
        let columns = columns.max(1);
        // Keep a cell for the insertion cursor, even when the prompt is wider
        // than the terminal. Query storage is never truncated by the viewport.
        let prompt = terminal::clip(&self.prompt, columns - 1);
        let available = columns - terminal::cell_width(prompt);
        let mut start = 0;
        let mut before_cursor: usize = self.query[..self.position].iter()
            .map(|c| unicode_width::UnicodeWidthChar::width(*c).unwrap_or(0)).sum();
        while before_cursor >= available && start < self.position {
            before_cursor -= unicode_width::UnicodeWidthChar::width(self.query[start]).unwrap_or(0);
            start += 1;
        }
        // Do not start a viewport with combining marks detached from their base.
        while start < self.position && unicode_width::UnicodeWidthChar::width(self.query[start]) == Some(0) {
            start += 1;
        }
        let visible: String = self.query[start..].iter().collect();
        let current_col = terminal::cell_width(prompt) + before_cursor + 1;
        format!("{}\r{}{}", cursor::clear_line(),
            cursor::nowrap(&format!("{}{}", prompt, terminal::clip(&visible, available))),
            cursor::col(current_col))
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

    pub fn delete(&mut self) -> Option<String> {
        if self.position != self.query.len() {
            self.query.remove(self.position);
            Some(self.draw())
        } else {
            None
        }
    }

    pub fn left(&mut self) -> Option<&str> {
        if self.position > 0 {
            self.position -= 1;
            self.redraw = self.draw();
            Some(&self.redraw)
        } else {
            None
        }
    }

    pub fn right(&mut self) -> Option<&str> {
        if self.query.len() != self.position {
            self.position += 1;
            self.redraw = self.draw();
            Some(&self.redraw)
        } else {
            None
        }
    }

    pub fn clear(&mut self) -> String {
        self.query = vec![];
        self.position = 0;
        self.draw()
    }

    pub fn left_word(&mut self) -> String {
        let regex = word_start_regex();
        let mut new_position = self.position;
        let query: String = self.query.iter().collect();

        regex
            .find_iter(&query)
            .for_each(|mat| {
                let start = query[..mat.start()].chars().count();
                if start < self.position {
                    new_position = start;
                }
            });
        self.position = new_position;

        self.draw()
    }

    pub fn right_word(&mut self) -> String {
        let regex = word_start_regex();
        let mut new_position = self.position;
        let query: String = self.query.iter().collect();

        regex
            .find_iter(&query)
            .for_each(|mat| {
                let start = query[..mat.start()].chars().count();
                if start > self.position && new_position == self.position {
                    new_position = start;
                }
            });
        if self.position == new_position {
            self.position = self.query.len();
        } else {
            self.position = new_position;
        }

        self.draw()
    }

    pub fn backspace_word(&mut self) -> String {
        let regex = word_start_regex();
        let mut new_position = self.position;
        let query: String = self.query.iter().collect();

        regex
            .find_iter(&query)
            .for_each(|mat| {
                let start = query[..mat.start()].chars().count();
                if start < self.position {
                    new_position = start;
                }
            });

        (new_position..self.position).rev().for_each(|i| {
            self.position -= 1;
            self.query.remove(i);
        });

        self.draw()
    }

    pub fn delete_word(&mut self) -> String {
        let regex = word_end_regex();
        let mut new_position = self.position;
        let query: String = self.query.iter().collect();

        regex
            .find_iter(&query)
            .for_each(|mat| {
                let start = query[..mat.start()].chars().count();
                if start > self.position && new_position == self.position {
                    new_position = query[..mat.end()].chars().count();
                }
            });
        if self.position == new_position {
            new_position = self.query.len();
        }
        (self.position..new_position).for_each(|_i| {
            self.query.remove(self.position);
        });

        self.draw()
    }

    pub fn set_query(&mut self, new_query: &str) -> String {
        self.query = new_query.chars().collect::<Vec<char>>();
        self.position = self.query.len();

        self.draw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_query_scrolls_without_changing_the_query() {
        let mut search = Search::new("> ");
        search.set_query("abcdefghijk");
        assert_eq!(search.draw_with_width(Some(8)), format!("{}\r{}{}",
            cursor::clear_line(), cursor::nowrap("> ghijk"), cursor::col(8)));
        search.position = 0;
        assert_eq!(search.draw_with_width(Some(8)), format!("{}\r{}{}",
            cursor::clear_line(), cursor::nowrap("> abcdef"), cursor::col(3)));
        assert_eq!(search.query.iter().collect::<String>(), "abcdefghijk");
    }

    #[test]
    fn unicode_completion_and_word_operations_use_character_positions() {
        let mut search = Search::new("> ");
        search.set_query("élan/東京/café");
        assert_eq!(search.position, 12);
        search.backspace();
        search.keypress('é');
        search.left_word();
        assert_eq!(search.position, 8);
        search.backspace_word();
        assert_eq!(search.position, 5);
        assert_eq!(search.query.iter().collect::<String>(), "élan/café");
        search.delete_word();
        assert_eq!(search.query.iter().collect::<String>(), "élan/");
    }

    #[test]
    fn viewport_counts_wide_and_combining_characters() {
        let mut search = Search::new("🔍 ");
        search.set_query("é東京e\u{301}");
        assert_eq!(search.draw_with_width(Some(8)), format!("{}\r{}{}",
            cursor::clear_line(), cursor::nowrap("🔍 京e\u{301}"), cursor::col(7)));
        assert_eq!(search.draw_with_width(Some(1)), format!("{}\r{}{}",
            cursor::clear_line(), cursor::nowrap(""), cursor::col(1)));
    }

    #[test]
    fn every_cursor_position_fits_even_in_tiny_windows() {
        for prompt in ["> ", "🔍 prompt "] {
            let mut search = Search::new(prompt);
            search.set_query("a東京e\u{301}💣/long/path");
            let original = search.query.clone();
            for width in 1..=12 {
                for position in 0..=search.query.len() {
                    search.position = position;
                    let drawn = search.draw_with_width(Some(width));
                    let (_, after) = drawn.split_once("\x1b[?7l").unwrap();
                    let (visible, cursor) = after.split_once("\x1b[?7h").unwrap();
                    assert!(terminal::cell_width(visible) <= width);
                    let column: usize = cursor.strip_prefix("\x1b[").unwrap()
                        .strip_suffix('G').unwrap().parse().unwrap();
                    assert!((1..=width).contains(&column));
                    assert_eq!(search.query, original);
                }
            }
        }
    }

    #[test]
    fn word_regexes_are_reused() {
        assert!(std::ptr::eq(word_start_regex(), word_start_regex()));
        assert!(std::ptr::eq(word_end_regex(), word_end_regex()));
    }

    #[test]
    fn word_regexes_preserve_unicode_boundaries() {
        let query = "élan 東京 e\u{301}_x 42";
        let starts: Vec<_> = word_start_regex()
            .find_iter(query)
            .map(|mat| (mat.start(), mat.as_str()))
            .collect();
        let ends: Vec<_> = word_end_regex()
            .find_iter(query)
            .map(|mat| (mat.start(), mat.as_str()))
            .collect();

        assert_eq!(starts, vec![(0, "é"), (6, "東"), (13, "e"), (19, "4")]);
        assert_eq!(ends, vec![(4, "n"), (9, "京"), (17, "x"), (20, "2")]);
    }

    #[test]
    fn word_operations_handle_empty_and_punctuation_queries() {
        for query in ["", " /.- "] {
            let mut search = Search::new("> ");
            search.set_query(query);
            search.left_word();
            assert_eq!(query.len(), search.position);
            search.backspace_word();
            assert_eq!(query, search.query.iter().collect::<String>());
            search.position = 0;
            search.right_word();
            assert_eq!(query.len(), search.position);
            search.position = 0;
            search.delete_word();
            assert!(search.query.is_empty());
        }
    }

    #[test]
    fn word_operations_reuse_patterns_across_searches() {
        for _ in 0..3 {
            let mut search = Search::new("> ");
            search.set_query("one/two_three 42");
            search.left_word();
            assert_eq!(14, search.position);
            search.left_word();
            assert_eq!(4, search.position);
            search.right_word();
            assert_eq!(14, search.position);
            search.backspace_word();
            assert_eq!(4, search.position);
            assert_eq!("one/42", search.query.iter().collect::<String>());
            search.delete_word();
            assert_eq!("one/", search.query.iter().collect::<String>());
        }
    }

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
    fn test_delete() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;

        assert_eq!(
            format!("{}\r> ac{}", cursor::clear_line(), cursor::col(4)),
            search.delete().unwrap()
        );
        assert_eq!(1, search.position);
    }

    #[test]
    fn test_delete_none() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 3;

        assert!(search.delete().is_none());
        assert_eq!(3, search.position);
    }

    #[test]
    fn test_left() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 'b', 'c'];
        search.position = 1;

        let drawn = search.left().unwrap().to_string();
        assert_eq!(search.draw(), drawn);
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

        let drawn = search.right().unwrap().to_string();
        assert_eq!(search.draw(), drawn);
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

    #[test]
    fn test_left_word() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 's', ' ', 'a', 's', ' ', 'a', 's', ' '];
        search.position = 7;

        search.left_word();

        assert_eq!(6, search.position);

        search.position = 3;
        search.left_word();
        assert_eq!(0, search.position);
    }

    #[test]
    fn test_right_word() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 's', ' ', 'a', 's', ' ', 'a', 's', ' '];

        search.position = 0;
        search.right_word();

        assert_eq!(3, search.position);

        search.position = 6;
        search.right_word();
        assert_eq!(9, search.position);
    }

    #[test]
    fn backspace_word_test() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 's', ' ', 'a', 's', ' ', 'a', 's', ' '];

        search.position = 5;
        search.backspace_word();

        assert_eq!(3, search.position);
        assert_eq!(vec!['a', 's', ' ', ' ', 'a', 's', ' '], search.query);

        search.backspace_word();

        assert_eq!(0, search.position);
        assert_eq!(vec![' ', 'a', 's', ' '], search.query);
    }

    #[test]
    fn delete_word_test() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 's', ' ', 'a', 's', ' ', 'a', 's', ' ', 'a', 's', ' '];

        search.position = 3;
        search.delete_word();

        assert_eq!(3, search.position);
        assert_eq!(
            vec!['a', 's', ' ', ' ', 'a', 's', ' ', 'a', 's', ' '],
            search.query
        );
    }

    #[test]
    fn delete_word_at_end_test() {
        let mut search = Search::new("> ");
        search.query = vec!['a', 's', ' ', 'a', 's', ' ', 'a', 's', ' ', 'a', 's', ' '];

        search.position = 10;
        search.delete_word();

        assert_eq!(10, search.position);
        assert_eq!(
            vec!['a', 's', ' ', 'a', 's', ' ', 'a', 's', ' ', 'a'],
            search.query
        );
    }

    #[test]
    fn set_query_test() {
        let mut search = Search::new("> ");
        search.query = vec!['a'];

        search.position = 1;
        search.set_query("foo");

        assert_eq!(3, search.position);
        assert_eq!(vec!['f', 'o', 'o'], search.query);
    }
}
