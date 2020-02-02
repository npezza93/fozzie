use crate::choice;
use std::fmt;

pub struct Match<'a> {
    pub choice: &'a str,
    score: f64,
    highlights: Vec<usize>,
}

impl<'a> Match<'a> {
    pub fn new(query: &[char], choice: &'a str) -> Option<Self> {
        if Self::matches(query, choice) {
            Some(Self {
                choice,
                score: 1.0,
                highlights: vec![],
            })
        } else {
            None
        }
    }

    pub fn draw(&self, selected: bool) -> String {
        if selected {
            color::inverse(&self.choice)
        } else {
            self.choice.into()
        }
    }

    fn matches(query: &[char], choice: &str) -> bool {
        query.iter().all(|&nchar| {
            choice.chars().any(|cchar| cchar.eq_ignore_ascii_case(&nchar))
        })
    }
}

impl<'a> From<&Match<'a>> for String {
    fn from(matcher: &Match<'a>) -> Self {
        matcher.choice.to_string()
    }
}

impl<'a> fmt::Display for Match<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.choice)
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
}
