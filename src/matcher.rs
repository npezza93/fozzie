use crate::color;
use crate::scorer::{Score, MIN};
use std::cmp::Ordering;
use std::fmt;

pub struct Match<'a> {
    pub choice: &'a str,
    query: String,
    scorer: Score,
}

impl<'a> Match<'a> {
    fn is_match(query: &[char], choice: &str) -> bool {
        // Saving the enumerator outside the iterator will ensure chars are in
        // order and will make it so we only ever go through the choice once.
        let mut choice_chars = choice.chars();

        query.iter().all(|nchar| {
            choice_chars.any(|cchar| cchar.eq_ignore_ascii_case(&nchar))
        })
    }

    pub fn new(query: &[char], choice: &'a str) -> Option<Self> {
        if Self::is_match(&query, &choice) {
            Some(Self {
                choice,
                query: query.iter().collect(),
                scorer: Score::new(&query, &choice),
            })
        } else {
            None
        }
    }

    pub fn draw(&self, selected: bool, show_scores: bool) -> String {
        let mut drawn = self.draw_highlights();

        if show_scores {
            let current_score = self.score();
            if current_score != MIN {
                drawn = format!("({:5.2}) {}", current_score, drawn);
            } else {
                drawn = format!("(     ) {}", drawn);
            }
        }

        if selected {
            color::inverse(&drawn)
        } else {
            drawn
        }
    }

    fn score(&self) -> f64 {
        self.scorer.score()
    }

    fn positions(&self) -> Vec<usize> {
        self.scorer.positions()
    }

    fn draw_highlights(&self) -> String {
        self.choice
            .chars()
            .enumerate()
            .map(|(i, cchar)| {
                if self.positions().contains(&i) {
                    color::highlight(cchar)
                } else {
                    cchar.to_string()
                }
            })
            .collect()
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
        other.score().partial_cmp(&self.score())
    }
}

impl<'a> Eq for Match<'a> {}
impl<'a> PartialEq for Match<'a> {
    fn eq(&self, other: &Self) -> bool {
        other.score() == self.score()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_match() {
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
        assert!(!Match::new(&['a', 'b', 'c'], "cab").is_some());
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

        assert_eq!("foo", matcher.draw(false, false));
    }

    #[test]
    fn test_draw_selected() {
        let matcher = Match::new(&[], "foo").unwrap();

        assert_eq!("\x1B[7mfoo\x1B[27m", matcher.draw(true, false));
    }

    #[test]
    fn test_drawing_unselected_highlights() {
        let matcher = Match::new(&['f'], "foo").unwrap();

        assert_eq!("\x1B[35mf\x1B[39moo", matcher.draw(false, false));
    }

    #[test]
    fn test_drawing_selected_highlights() {
        let matcher = Match::new(&['f'], "foo").unwrap();

        assert_eq!("\x1B[7m\x1B[35mf\x1B[39moo\x1B[27m", matcher.draw(true, false));
    }

    #[test]
    fn drawing_with_show_scores_empty_test() {
        let matcher = Match::new(&[], "foo").unwrap();

        assert_eq!("(     ) foo", matcher.draw(false, true))
    }

    #[test]
    fn drawing_with_show_scores_test() {
        let matcher = Match::new(&['f'], "foo").unwrap();

        assert_eq!("( 0.89) \u{1b}[35mf\u{1b}[39moo", matcher.draw(false, true))
    }
}
