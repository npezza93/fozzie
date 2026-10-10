use crate::choice::Choice;
use crate::cursor;
use crate::matcher::Match;
use crate::scorer::ScoreBuffer;
use crate::terminal::Terminal;
use rayon::prelude::*;
use std::collections::BTreeMap;

pub struct Choices<'a> {
    choices: &'a [Choice<'a>],
    selected: usize,
    max_choices: usize,
    matches: Vec<RankedChoice<'a>>,
    query: Vec<char>,
    visible_matches: BTreeMap<usize, Match<'a>>,
    show_scores: bool,
}

// Keep every ranked candidate available for scrolling without allocating its
// highlight positions. Only the visible range needs a full Match.
struct RankedChoice<'a> {
    choice: &'a Choice<'a>,
    score: f32,
}

impl<'a> Choices<'a> {
    const OFFSET: usize = 1;

    pub fn new(max_choices: usize, choices: &'a [Choice], show_scores: bool) -> Choices<'a> {
        Choices {
            selected: 0,
            matches: vec![],
            query: vec![],
            visible_matches: BTreeMap::new(),
            choices,
            max_choices,
            show_scores,
        }
    }

    pub fn initial_draw(&mut self, terminal: &mut Terminal) {
        self.filter_choices(&[]);

        terminal.print(&format!(
            "\r\n{}{}\r",
            self.draw_choices(),
            cursor::up(self.max_choices())
        ));
    }

    pub fn previous(&mut self) -> String {
        if self.selected == 0 {
            self.selected = self.last_index();
        } else {
            self.selected -= 1;
        }

        self.draw()
    }

    pub fn next(&mut self) -> String {
        if self.selected == self.last_index() {
            self.selected = 0;
        } else {
            self.selected += 1;
        }

        self.draw()
    }

    pub fn select(&self, terminal: &mut Terminal) {
        terminal.print(&format!("\r{}", cursor::clear_screen_down()));
        if !self.matches.is_empty() {
            println!("{}", self.current_match().returnable);
        }
    }

    pub fn current_match(&self) -> &Choice {
        self.matches[self.selected].choice
    }

    pub fn cancel(&self) -> String {
        format!("\r{}", cursor::clear_screen_down())
    }

    pub fn filter(&mut self, query: &[char]) -> String {
        self.filter_choices(query);
        self.draw()
    }

    fn filter_choices(&mut self, query: &[char]) {
        self.selected = 0;
        self.visible_matches.clear();
        self.query.clear();
        self.query.extend_from_slice(query);
        let lower_query: Vec<char> = query.iter().map(|c| c.to_ascii_lowercase()).collect();
        self.matches = self
            .choices
            .par_iter()
            .map_init(ScoreBuffer::default, |buffer, choice| {
                if Match::is_match(query, choice) {
                    Some(RankedChoice {
                        choice,
                        score: buffer.score(&lower_query, choice),
                    })
                } else {
                    None
                }
            })
            .filter_map(|matched| matched)
            .collect();
        // Stable ordering keeps equal-score choices in their original order.
        self.matches
            .par_sort_by(|left, right| right.score.partial_cmp(&left.score).unwrap());
    }

    fn draw(&mut self) -> String {
        format!(
            "{}\r\n{}\r{}{}",
            cursor::save_position(),
            cursor::clear_screen_down(),
            self.draw_choices(),
            cursor::restore_position(),
        )
    }

    fn last_index(&self) -> usize {
        if self.matches.is_empty() {
            0
        } else {
            self.matches.len() - 1
        }
    }

    fn draw_choices(&mut self) -> String {
        let range = self.drawn_range();
        // Keep highlights only for the current viewport. Overlapping rows can
        // be reused on arrow-key navigation; new rows are reconstructed lazily.
        self.visible_matches
            .retain(|index, _| range.contains(index));
        range
            .map(|i| {
                self.visible_matches
                    .entry(i)
                    .or_insert_with(|| {
                        Match::new(&self.query, self.matches[i].choice)
                            .expect("ranked choices match the current query")
                    })
                    .draw(i == self.selected, self.show_scores)
            })
            .collect::<Vec<String>>()
            .join("\n\r")
    }

    fn max_choices(&self) -> usize {
        if self.matches.len() < self.max_choices {
            self.matches.len()
        } else {
            self.max_choices
        }
    }

    fn drawn_range(&self) -> std::ops::Range<usize> {
        let index = self.starting_position();

        index..(index + self.max_choices())
    }

    fn starting_position(&self) -> usize {
        if self.selected + Self::OFFSET < self.max_choices {
            0
        } else if self.selected + Self::OFFSET + 1 >= self.matches.len() {
            self.matches.len() - self.max_choices
        } else {
            self.selected + Self::OFFSET + 1 - self.max_choices
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;
    use crate::config::Config;

    #[test]
    fn test_new() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
            make_choice("baz"),
            make_choice("boo"),
        ];
        let choices = Choices::new(4, &input, false);

        assert_eq!(4, choices.max_choices);
        assert_eq!(0, choices.selected);
        assert_eq!(
            vec!["foo", "bar", "baz", "boo"],
            choices.choices.iter().map(|choice| choice.searchable).collect::<Vec<&str>>(),
        );
    }

    #[test]
    fn test_new_max_choices() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
            make_choice("baz"),
            make_choice("boo"),
        ];
        let choices = Choices::new(2, &input, false);

        assert_eq!(2, choices.max_choices);
    }

    #[test]
    fn test_filter() {
        let input: Vec<Choice> = vec![
            make_choice("foo"), make_choice("bar"),
        ];
        let mut choices = Choices::new(4, &input, false);

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\r{}{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                cursor::nowrap(&color::inverse("foo")),
                cursor::nowrap("bar"),
                cursor::restore_position()
            ),
            choices.filter(&[])
        );
        assert_eq!(
            vec!["foo", "bar"],
            choices
                .matches
                .iter()
                .map(|matcher| matcher.choice.searchable)
                .collect::<Vec<&str>>()
        );
    }

    #[test]
    fn test_previous_when_wrapping() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
        ];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\r{}{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                cursor::nowrap("foo"),
                cursor::nowrap(&color::inverse("bar")),
                cursor::restore_position(),
            ),
            choices.previous()
        );
        assert_eq!(1, choices.selected);
    }

    #[test]
    fn test_previous() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
        ];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);
        choices.selected = 1;

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\r{}{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                cursor::nowrap(&color::inverse("foo")),
                cursor::nowrap("bar"),
                cursor::restore_position(),
            ),
            choices.previous()
        );
        assert_eq!(0, choices.selected);
    }

    #[test]
    fn test_next_when_wrapping() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
        ];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);
        choices.selected = 1;

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\r{}{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                cursor::nowrap(&color::inverse("foo")),
                cursor::nowrap("bar"),
                cursor::restore_position(),
            ),
            choices.next()
        );
        assert_eq!(0, choices.selected);
    }

    #[test]
    fn test_next() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
        ];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\r{}{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                cursor::nowrap("foo"),
                cursor::nowrap(&color::inverse("bar")),
                cursor::restore_position(),
            ),
            choices.next()
        );
        assert_eq!(1, choices.selected);
    }

    #[test]
    fn test_cancel() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
        ];
        let choices = Choices::new(4, &input, false);

        assert_eq!(
            format!("\r{}", cursor::clear_screen_down()),
            choices.cancel()
        );
    }

    #[test]
    fn test_current_match() {
        let input: Vec<Choice> = vec![
            make_choice("foo"),
            make_choice("bar"),
        ];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);

        assert_eq!("foo", choices.current_match().searchable);
    }

    #[test]
    fn score_only_ranking_and_scrolling_match_eager_matches() {
        let input: Vec<Choice> = vec![
            make_choice("src/class.rs"),
            make_choice("src/classes.rs"),
            make_choice("src/Class.rs"),
            make_choice("src/class.rs"),
            make_choice("class"),
            make_choice("README.md"),
            make_choice("src/search.rs"),
            make_choice("src/scorer.rs"),
        ];
        let mut choices = Choices::new(3, &input, true);

        for query in ["src", "class", "Class", "", "no match", "s"] {
            let query: Vec<char> = query.chars().collect();
            let mut eager: Vec<Match> = input
                .iter()
                .filter_map(|choice| Match::new(&query, choice))
                .collect();
            eager.sort();
            choices.filter(&query);
            assert_eq!(choices.visible_matches.len(), choices.max_choices());
            assert_eq!(choices.query, query);
            assert_eq!(choices.matches.len(), eager.len());
            for (ranked, original) in choices.matches.iter().zip(&eager) {
                assert!(std::ptr::eq(ranked.choice, original.choice));
                assert_eq!(
                    ranked.score.to_bits(),
                    crate::scorer::Score::new(&query, original.choice)
                        .score
                        .to_bits()
                );
            }

            // Visit every result, wrap, then traverse backwards. This also
            // verifies highlights and displayed scores outside the first page.
            for forward in [true, false] {
                for _ in 0..=eager.len() {
                    let expected = choices
                        .drawn_range()
                        .map(|i| eager[i].draw(i == choices.selected, true))
                        .collect::<Vec<String>>()
                        .join("\n\r");
                    assert_eq!(choices.draw_choices(), expected);
                    assert_eq!(choices.visible_matches.len(), choices.max_choices());
                    assert!(choices
                        .visible_matches
                        .keys()
                        .all(|i| choices.drawn_range().contains(i)));
                    if forward {
                        choices.next();
                    } else {
                        choices.previous();
                    }
                }
            }
        }
    }

    #[test]
    fn score_only_ranking_preserves_unicode_and_uppercase_matching() {
        let input = vec![
            make_choice("é/λabc"),
            make_choice("é/λAbc"),
            make_choice("λabc"),
            make_choice("é/λabc"),
        ];
        let mut choices = Choices::new(2, &input, false);
        for query in ["λa", "λA", "é", "", "z"] {
            let query: Vec<char> = query.chars().collect();
            choices.filter_choices(&query);
            let mut eager: Vec<Match> = input
                .iter()
                .filter_map(|choice| Match::new(&query, choice))
                .collect();
            eager.sort();
            assert_eq!(choices.matches.len(), eager.len());
            for (ranked, original) in choices.matches.iter().zip(&eager) {
                assert!(std::ptr::eq(ranked.choice, original.choice));
            }
        }
    }

    #[bench]
    fn bench_filtering(b: &mut test::Bencher) {
        let choices: Vec<Choice> = vec![
            make_choice("CODE_OF_CONDUCT.md"),
            make_choice("Cargo.lock"),
            make_choice("Cargo.toml"),
            make_choice("LICENSE"),
            make_choice("README.md"),
            make_choice("benches/choices.rs"),
            make_choice("benches/drawing.rs"),
            make_choice("benches/matching.rs"),
            make_choice("benches/scoring.rs"),
            make_choice("src/bonus.rs")
        ];
        let query = ['c', 'o', 'd', 'e'];

        b.iter(|| Choices::new(10, &choices, false).filter(&query))
    }

    fn make_choice(choice: &str) -> Choice {
        let config = Config {
            lines: 10, prompt: ">".to_string(), show_scores: false,
            query: None, delimiter: None, field: None, output: None,
            benchmark: false,
        };

        Choice::new(choice, &config)
    }
}
