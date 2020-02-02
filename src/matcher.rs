use crate::color;
use std::cmp::Ordering;
use std::fmt;

pub struct Match<'a> {
    pub choice: &'a str,
    highlights: Vec<usize>,
    score: f64,
}

impl<'a> Match<'a> {
    pub fn new(query: &[char], choice: &'a str) -> Option<Self> {
        let mut matcher = Self {
            choice,
            highlights: vec![],
            score: 1.0,
        };

        if matcher.matches(query) {
            matcher.set_score(&query);
            Some(matcher)
        } else {
            None
        }
    }

    pub fn draw(&self, selected: bool) -> String {
        if selected {
            color::inverse(&self.draw_highlights())
        } else {
            self.draw_highlights()
        }
    }

    fn matches(&mut self, query: &[char]) -> bool {
        query.iter().all(|&nchar| {
            self.choice.chars().enumerate().any(|(i, cchar)| {
                if cchar.eq_ignore_ascii_case(&nchar) {
                    self.highlights.push(i);
                    true
                } else {
                    false
                }
            })
        })
    }

    fn draw_highlights(&self) -> String {
        self.choice.chars().enumerate().map(|(i, cchar)| {
            if self.highlights.contains(&i) {
                color::highlight(cchar)
            } else {
                cchar.to_string()
            }
        }).collect()
    }

    fn set_score(&mut self, _query: &[char]) {
        self.score = 1.0;
    }
}

impl<'a> fmt::Display for Match<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.choice)
    }
}

impl<'a> Ord for Match<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl<'a> PartialOrd for Match<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

impl<'a> Eq for Match<'a> {}
impl<'a> PartialEq for Match<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(Match::new(&['a'], "a").is_some());
        assert!(Match::new(&['a'], "abc").is_some());
        assert!(Match::new(&['a', 'b', 'c'], "abc").is_some());
        assert!(Match::new(&['A', 'B', 'C'], "abc").is_some());
        assert!(Match::new(&['a', 'b', 'c'], "a1b2c3").is_some());
        assert!(Match::new(&['a', 'b', 'c'], "a1b2c3").is_some());
        assert!(Match::new(&['t', 'e', 's', 't'], "t/e/s/t").is_some());
        assert!(Match::new(&['t', 'e', 's', 't'], "t💣e💣s💣t").is_some());
        assert!(Match::new(&['💣', '💣', '💣'], "t💣e💣s💣t").is_some());

        assert!(!Match::new(&['a', 'b', 'c'], "ab").is_some());
        assert!(!Match::new(&['a', 'b', 'c'], "").is_some());

        assert!(Match::new(&[], "").is_some());
        assert!(Match::new(&[], "ab").is_some());

        // UTF-8 case testing
        assert!(Match::new(&['a'], "A").is_some());
        assert!(Match::new(&['A'], "a").is_some());
    }

    #[test]
    fn test_draw_not_selected() {
        let matcher = Match::new(&[], "foo").unwrap();

        assert_eq!("foo", matcher.draw(false));
    }

    #[test]
    fn test_draw_selected() {
        let matcher = Match::new(&[], "foo").unwrap();

        assert_eq!("\x1B[7mfoo\x1B[27m", matcher.draw(true));
    }

    #[test]
    fn test_drawing_unselected_highlights() {
        let mut matcher = Match::new(&['f'], "foo").unwrap();
        matcher.highlights = vec![0];

        assert_eq!("\x1B[33mf\x1B[39moo", matcher.draw(false));
    }

    #[test]
    fn test_drawing_selected_highlights() {
        let mut matcher = Match::new(&['f'], "foo").unwrap();
        matcher.highlights = vec![0];

        assert_eq!("\x1B[7m\x1B[33mf\x1B[39moo\x1B[27m", matcher.draw(true));
    }
}
