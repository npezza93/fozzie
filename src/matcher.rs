use crate::color;
use crate::scorer::{Score, MIN};
use std::cmp::Ordering;
use std::fmt;

pub struct Match<'a> {
    pub choice: &'a str,
    highlights: Vec<usize>,
    scorer: Option<Score>,
}

impl<'a> Match<'a> {
    pub fn new(query: &[char], choice: &'a str) -> Option<Self> {
        let mut matcher = Self {
            choice,
            highlights: vec![],
            scorer: None,
        };

        if matcher.matches(query) {
            matcher.scorer = Some(Score::new(&query, &choice));
            Some(matcher)
        } else {
            None
        }
    }

    pub fn draw(&self, selected: bool, show_scores: bool) -> String {
        let mut drawn =
            if selected {
                color::inverse(&self.draw_highlights())
            } else {
                self.draw_highlights()
            };

        let current_score = self.scorer.as_ref().unwrap().score();
        if show_scores && current_score != MIN {
            drawn = format!("({:5.2}) {}", current_score, drawn);
        } else if show_scores {
            drawn = format!("(     ) {}", drawn);
        }

        drawn
    }

    fn matches(&mut self, query: &[char]) -> bool {
        // Saving the enumerator outside the iterator will ensure chars are in
        // order and will make it so we only ever go through the choice once.
        let mut choice_chars = self.choice.chars();

        query.iter().all(|&nchar| {
            choice_chars.any(|cchar| cchar.eq_ignore_ascii_case(&nchar))
        })
    }

    fn draw_highlights(&self) -> String {
        self.choice
            .chars()
            .enumerate()
            .map(|(i, cchar)| {
                if self.highlights.contains(&i) {
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
        let other_scorer = other.scorer.as_ref().unwrap();
        let current_scorer = self.scorer.as_ref().unwrap();

        other_scorer.score().partial_cmp(&current_scorer.score())
    }
}

impl<'a> Eq for Match<'a> {}
impl<'a> PartialEq for Match<'a> {
    fn eq(&self, other: &Self) -> bool {
        let other_scorer = other.scorer.as_ref().unwrap();
        let current_scorer = self.scorer.as_ref().unwrap();

        current_scorer.score() == other_scorer.score()
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
        let mut matcher = Match::new(&['f'], "foo").unwrap();
        matcher.highlights = vec![0];

        assert_eq!("\x1B[33mf\x1B[39moo", matcher.draw(false, false));
    }

    #[test]
    fn test_drawing_selected_highlights() {
        let mut matcher = Match::new(&['f'], "foo").unwrap();
        matcher.highlights = vec![0];

        assert_eq!("\x1B[7m\x1B[33mf\x1B[39moo\x1B[27m", matcher.draw(true, false));
    }

    #[test]
    fn drawing_with_show_scores_empty_test() {
        let matcher = Match::new(&[], "foo").unwrap();

        assert_eq!("(     ) foo", matcher.draw(false, true))
    }

    #[test]
    fn drawing_with_show_scores_test() {
        let mut matcher = Match::new(&['f'], "foo").unwrap();
        matcher.highlights = vec![];

        assert_eq!("( 0.89) foo", matcher.draw(false, true))
    }
}
