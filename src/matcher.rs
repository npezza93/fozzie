use crate::choice::Choice;
use crate::color;
use crate::cursor;
use crate::scorer::{Score, MIN};
use std::cmp::Ordering;
use std::fmt;

use terminal_size::{Width, terminal_size_of};

pub struct Match<'a> {
    pub choice: &'a Choice<'a>,
    scorer: Score,
}

impl<'a> Match<'a> {
    pub fn is_match(query: &[char], choice: &Choice) -> bool {
        // Saving the enumerator outside the iterator will ensure chars are in
        // order and will make it so we only ever go through the choice once.
        let mut choice_chars = choice.searchable.chars();

        query.iter().all(|nchar| {
            choice_chars.any(|cchar| {
                nchar == &cchar || nchar.to_ascii_uppercase() == cchar
            })
        })
    }

    pub fn new(query: &[char], choice: &'a Choice) -> Option<Self> {
        if Self::is_match(query, choice) {
            Some(Self {
                choice,
                scorer: Score::new(query, choice),
            })
        } else {
            None
        }
    }

    pub fn draw(&self, selected: bool, show_scores: bool) -> String {
        let mut drawn = self.draw_highlights();

        if show_scores {
            let current_score = self.scorer.score;
            if current_score != MIN {
                drawn = format!("({:5.2}) {}", current_score, drawn);
            } else {
                drawn = format!("(     ) {}", drawn);
            }
        }

        if selected {
            cursor::nowrap(&color::inverse(&drawn))
        } else {
            cursor::nowrap(&drawn)
        }
    }

    fn draw_highlights(&self) -> String {
        // Preserve stdout-only sizing when selected output is piped elsewhere.
        let limit = terminal_size_of(std::io::stdout())
            .map(|(Width(width), _)| usize::from(width));
        self.draw_highlights_with_limit(limit)
    }

    fn draw_highlights_with_limit(&self, limit: Option<usize>) -> String {
        // Scores use character positions, and the terminal limit must not be
        // used as a byte offset into UTF-8 text.
        self.choice.searchable.chars().take(limit.unwrap_or(usize::MAX))
            .enumerate()
            .map(|(i, cchar)| {
                if self.scorer.positions.contains(&i) {
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
        write!(f, "{}", self.choice.searchable)
    }
}

impl<'a> Ord for Match<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.scorer.score.partial_cmp(&self.scorer.score).unwrap()
    }
}

impl<'a> PartialOrd for Match<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Eq for Match<'a> {}
impl<'a> PartialEq for Match<'a> {
    fn eq(&self, other: &Self) -> bool {
        other.scorer.score == self.scorer.score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn score_ordering_is_descending_and_consistent() {
        let choice = make_choice("foo");
        let scores = [f32::NEG_INFINITY, -1.0, -0.0, 0.0, 1.0, f32::INFINITY];
        for left_score in scores {
            for right_score in scores {
                let left = Match {
                    choice: &choice,
                    scorer: Score { score: left_score, positions: vec![] },
                };
                let right = Match {
                    choice: &choice,
                    scorer: Score { score: right_score, positions: vec![] },
                };
                let expected = right_score.partial_cmp(&left_score).unwrap();
                assert_eq!(left.cmp(&right), expected);
                assert_eq!(left.partial_cmp(&right), Some(expected));
                assert_eq!(left == right, expected == Ordering::Equal);
            }
        }
    }

    #[test]
    fn test_is_match() {
        assert!(new_match("a", &make_choice("a")).is_some());
        assert!(new_match("a", &make_choice("abc")).is_some());
        assert!(new_match("abc", &make_choice("abc")).is_some());
        assert!(new_match("ABC", &make_choice("abc")).is_none());
        assert!(new_match("abc", &make_choice("a1b2c3")).is_some());
        assert!(new_match("abc", &make_choice("a1b2c3")).is_some());
        assert!(new_match("test", &make_choice("t/e/s/t")).is_some());
        assert!(new_match("test", &make_choice("t💣e💣s💣t")).is_some());
        assert!(new_match("💣💣💣", &make_choice("t💣e💣s💣t")).is_some());

        assert!(new_match("abc", &make_choice("ab")).is_none());
        assert!(new_match("abc", &make_choice("cab")).is_none());
        assert!(new_match("abc", &make_choice("")).is_none());

        assert!(new_match("", &make_choice("")).is_some());
        assert!(new_match("", &make_choice("ab")).is_some());

        // UTF-8 case testing
        assert!(new_match("a", &make_choice("A")).is_some());
        assert!(new_match("A", &make_choice("a")).is_none());
    }

    #[test]
    fn test_draw_not_selected() {
        let choice = make_choice("foo");
        let matcher = new_match("", &choice).unwrap();

        assert_eq!("\x1B[?7lfoo\x1B[?7h", matcher.draw(false, false));
    }

    #[test]
    fn test_match_upper_case() {
        assert!(new_match("iT", &make_choice("iTunes")).is_some());
        assert!(new_match("it", &make_choice("iTunes")).is_some());
        assert!(new_match("It", &make_choice("iTunes")).is_none());
    }

    #[test]
    fn test_draw_selected() {
        let choice = make_choice("foo");
        let matcher = new_match("", &choice).unwrap();

        assert_eq!("\x1B[?7l\x1B[7mfoo\x1B[27m\x1B[?7h", matcher.draw(true, false));
    }

    #[test]
    fn test_drawing_unselected_highlights() {
        let choice = make_choice("foo");
        let matcher = new_match("f", &choice).unwrap();

        assert_eq!("\x1B[?7l\x1B[33mf\x1B[39moo\x1B[?7h", matcher.draw(false, false));
    }

    #[test]
    fn test_drawing_selected_highlights() {
        let choice = make_choice("foo");
        let matcher = new_match("f", &choice).unwrap();

        assert_eq!("\x1B[?7l\x1B[7m\x1B[33mf\x1B[39moo\x1B[27m\x1B[?7h", matcher.draw(true, false));
    }

    #[test]
    fn drawing_with_show_scores_empty_test() {
        let choice = make_choice("foo");
        let matcher = new_match("", &choice).unwrap();

        assert_eq!("\x1B[?7l(     ) foo\x1B[?7h", matcher.draw(false, true))
    }

    #[test]
    fn drawing_with_show_scores_test() {
        let choice = make_choice("foo");
        let matcher = new_match("f", &choice).unwrap();

        assert_eq!("\x1B[?7l( 0.99) \u{1b}[33mf\u{1b}[39moo\x1B[?7h", matcher.draw(false, true))
    }

    #[test]
    fn drawing_clips_characters_without_slicing_utf8() {
        for content in ["café", "東京", "a💣b", "e\u{301}lan", "src/東京/café.rs"] {
            let choice = make_choice(content);
            let matcher = new_match("", &choice).unwrap();
            for limit in 0..=content.chars().count() + 1 {
                assert_eq!(
                    matcher.draw_highlights_with_limit(Some(limit)),
                    content.chars().take(limit).collect::<String>(),
                    "{content:?} at limit {limit}",
                );
            }
            assert_eq!(matcher.draw_highlights_with_limit(None), content);
        }
    }

    #[test]
    fn clipping_preserves_character_based_highlights() {
        let choice = make_choice("é東京e\u{301}/file.rs");
        let matcher = new_match("京\u{301}f", &choice).unwrap();
        assert_eq!(matcher.draw_highlights_with_limit(Some(2)), "é東");
        assert_eq!(matcher.draw_highlights_with_limit(Some(3)),
                   format!("é東{}", color::highlight('京')));
        assert_eq!(matcher.draw_highlights_with_limit(Some(7)),
                   format!("é東{}e{}/{}", color::highlight('京'),
                           color::highlight('\u{301}'), color::highlight('f')));
    }

    #[test]
    fn ascii_clipping_and_empty_content_are_unchanged() {
        let choice = make_choice("some/long/path.rs");
        let matcher = new_match("s", &choice).unwrap();
        assert_eq!(matcher.draw_highlights_with_limit(Some(4)),
                   format!("{}ome", color::highlight('s')));
        assert_eq!(matcher.draw_highlights_with_limit(Some(0)), "");
        let empty = make_choice("");
        assert_eq!(new_match("", &empty).unwrap().draw_highlights_with_limit(Some(80)), "");
    }

    fn new_match<'a>(query: &str, choice: &'a Choice) -> Option<Match<'a>> {
        Match::new(&query.chars().collect::<Vec<char>>(), choice)
    }

    fn make_choice(choice: &str) -> Choice<'_> {
        let config = Config {
            lines: 10, prompt: ">".to_string(), show_scores: false,
            query: None, delimiter: None, field: None, output: None,
            benchmark: false,
        };

        Choice::new(choice, &config)
    }
}
