use crate::choice::Choice;
use crate::color;
use crate::cursor;
use crate::scorer::{Score, MIN};
use std::cmp::Ordering;
use std::fmt;

use crate::terminal;

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
        self.draw_with_limit(selected, show_scores, terminal::columns())
    }

    pub(crate) fn draw_with_limit(&self, selected: bool, show_scores: bool, limit: Option<usize>) -> String {
        self.draw_scrolled(selected, show_scores, limit, 0)
    }

    fn score_prefix(&self, show_scores: bool) -> String {
        if !show_scores {
            String::new()
        } else if self.scorer.score != MIN {
            format!("({:5.2}) ", self.scorer.score)
        } else {
            "(     ) ".to_string()
        }
    }

    pub(crate) fn max_scroll(&self, show_scores: bool, columns: usize) -> usize {
        let available = columns.saturating_sub(terminal::cell_width(&self.score_prefix(show_scores)));
        if available == 0 {
            0
        } else {
            terminal::cell_width(self.choice.searchable).saturating_sub(available)
        }
    }

    pub(crate) fn draw_scrolled(&self, selected: bool, show_scores: bool,
                                limit: Option<usize>, offset: usize) -> String {
        let prefix = self.score_prefix(show_scores);
        let prefix = terminal::clip(&prefix, limit.unwrap_or(usize::MAX));
        let remaining = limit.map(|width| width.saturating_sub(terminal::cell_width(prefix)));
        let drawn = prefix.to_string() + &self.draw_highlights_scrolled(remaining, offset);
        if selected {
            cursor::nowrap(&color::inverse(&drawn))
        } else {
            cursor::nowrap(&drawn)
        }
    }

    #[cfg(test)]
    fn draw_highlights_with_limit(&self, limit: Option<usize>) -> String {
        self.draw_highlights_scrolled(limit, 0)
    }

    fn draw_highlights_scrolled(&self, limit: Option<usize>, offset: usize) -> String {
        let mut drawn = String::new();
        let mut column: usize = 0;
        let end = offset.saturating_add(limit.unwrap_or(usize::MAX));
        let mut visible_base = false;
        // Keep original character indices for highlighting. A wide character
        // cut by the left edge leaves blank cells, never half a glyph; combining
        // marks are only drawn when their base is visible.
        for (i, character) in self.choice.searchable.chars().enumerate() {
            let width = unicode_width::UnicodeWidthChar::width(character).unwrap_or(0);
            if width == 0 {
                if !visible_base {
                    continue;
                }
            } else {
                let next = column + width;
                if next > end {
                    break;
                }
                visible_base = column >= offset;
                if !visible_base {
                    if next > offset {
                        drawn.push_str(&" ".repeat(next - offset));
                    }
                    column = next;
                    continue;
                }
                column = next;
            }
            if self.scorer.positions.contains(&i) {
                drawn.push_str(&color::highlight(character));
            } else {
                drawn.push(character);
            }
        }
        drawn
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
    fn drawing_clips_cells_without_slicing_utf8() {
        for content in ["café", "東京", "a💣b", "e\u{301}lan", "src/東京/café.rs"] {
            let choice = make_choice(content);
            let matcher = new_match("", &choice).unwrap();
            for limit in 0..=content.chars().count() + 1 {
                assert_eq!(
                    matcher.draw_highlights_with_limit(Some(limit)),
                    terminal::clip(content, limit),
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
        assert_eq!(matcher.draw_highlights_with_limit(Some(3)), "é東");
        assert_eq!(matcher.draw_highlights_with_limit(Some(5)),
                   format!("é東{}", color::highlight('京')));
        assert_eq!(matcher.draw_highlights_with_limit(Some(8)),
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

    #[test]
    fn score_prefix_is_part_of_the_row_budget() {
        let choice = make_choice("long/path.svg");
        let matcher = new_match("", &choice).unwrap();
        assert_eq!(matcher.draw_with_limit(false, true, Some(10)), cursor::nowrap("(     ) lo"));
        assert_eq!(matcher.draw_with_limit(false, true, Some(3)), cursor::nowrap("(  "));
    }

    #[test]
    fn scrolling_preserves_unicode_cells_and_original_highlight_indices() {
        let choice = make_choice("ab東京e\u{301}/filename.rs");
        let matcher = new_match("京\u{301}f", &choice).unwrap();
        assert_eq!(matcher.draw_highlights_scrolled(Some(8), 1),
            format!("b東{}e{}/{}", color::highlight('京'),
                    color::highlight('\u{301}'), color::highlight('f')));
        assert_eq!(matcher.draw_highlights_scrolled(Some(8), 3),
            format!(" {}e{}/{}il", color::highlight('京'),
                    color::highlight('\u{301}'), color::highlight('f')));
        assert_eq!(matcher.draw_highlights_scrolled(Some(5), 7),
            format!("/{}ile", color::highlight('f')));
        assert_eq!(matcher.draw_highlights_scrolled(Some(1), 3), " ");
        assert_eq!(matcher.draw_highlights_scrolled(Some(0), 3), "");
        assert_eq!(choice.searchable, "ab東京e\u{301}/filename.rs");
    }

    #[test]
    fn scrolling_keeps_scores_fixed_and_inside_the_row_budget() {
        let choice = make_choice("abcdefghijk");
        let matcher = new_match("", &choice).unwrap();
        assert_eq!(matcher.max_scroll(true, 12), 7);
        assert_eq!(matcher.max_scroll(true, 8), 0);
        assert_eq!(matcher.max_scroll(false, 12), 0);
        assert_eq!(matcher.draw_scrolled(false, true, Some(12), 2),
            cursor::nowrap("(     ) cdef"));
        assert_eq!(matcher.draw_scrolled(true, true, Some(3), 2),
            cursor::nowrap(&color::inverse("(  ")));
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
