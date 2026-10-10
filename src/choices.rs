use rayon::prelude::*;
use crate::choice::Choice;
use crate::cursor;
use crate::matcher::Match;
use crate::terminal::Terminal;

pub struct Choices<'a> {
    choices: &'a [Choice<'a>],
    selected: usize,
    max_choices: usize,
    matches: Vec<Match<'a>>,
    show_scores: bool,
    horizontal_offset: usize,
}

impl<'a> Choices<'a> {
    const OFFSET: usize = 1;

    pub fn new(max_choices: usize, choices: &'a [Choice], show_scores: bool) -> Choices<'a> {
        Choices {
            selected: 0,
            horizontal_offset: 0,
            matches: vec![],
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

    /// Move the selection forward and redraw, wrapping at the last match.
    /// Kept for existing library callers; prefer `select_next` in new code.
    // Cyclic UI navigation returns a redraw String, not Iterator::next's Option<Item>.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> String {
        self.select_next()
    }

    pub fn select_next(&mut self) -> String {
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

    pub(crate) fn selected_choice(&self) -> Option<&Choice<'_>> {
        self.matches.get(self.selected).map(|matched| matched.choice)
    }

    pub fn current_match(&self) -> &Choice<'_> {
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
        self.horizontal_offset = 0;
        self.matches = self
            .choices
            .par_iter()
            .filter_map(|choice| Match::new(query, choice))
            .collect::<Vec<Match<'a>>>();
        self.matches.par_sort();
    }

    fn draw(&self) -> String {
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

    pub(crate) fn scroll_right(&mut self) -> String {
        self.shift_horizontal(true, crate::terminal::columns());
        self.draw()
    }

    pub(crate) fn scroll_left(&mut self) -> String {
        self.shift_horizontal(false, crate::terminal::columns());
        self.draw()
    }

    fn max_horizontal_offset(&self, columns: Option<usize>) -> usize {
        columns.map_or(0, |columns| self.drawn_range()
            .map(|i| self.matches[i].max_scroll(self.show_scores, columns))
            .max().unwrap_or(0))
    }

    fn shift_horizontal(&mut self, right: bool, columns: Option<usize>) {
        let maximum = self.max_horizontal_offset(columns);
        let current = self.horizontal_offset.min(maximum);
        self.horizontal_offset = if right {
            current.saturating_add(1).min(maximum)
        } else {
            current.saturating_sub(1)
        };
    }

    fn draw_choices(&self) -> String {
        self.draw_choices_with_width(crate::terminal::columns())
    }

    fn draw_choices_with_width(&self, columns: Option<usize>) -> String {
        // Resizing or navigating to shorter rows must not leave a blank viewport.
        let offset = if self.horizontal_offset == 0 {
            0
        } else {
            self.horizontal_offset.min(self.max_horizontal_offset(columns))
        };
        self.drawn_range()
            .map(|i| self.matches[i].draw_scrolled(i == self.selected, self.show_scores, columns, offset))
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
            choices.select_next()
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
            choices.select_next()
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
    fn next_alias_preserves_selection_and_wrapping() {
        let input = vec![make_choice("foo"), make_choice("bar")];
        let mut legacy = Choices::new(1, &input, false);
        let mut explicit = Choices::new(1, &input, false);
        legacy.filter(&[]);
        explicit.filter(&[]);
        for _ in 0..4 {
            assert_eq!(legacy.next(), explicit.select_next());
            assert_eq!(legacy.current_match().searchable, explicit.current_match().searchable);
        }
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
    fn horizontal_scrolling_is_bounded_and_filtering_resets_it() {
        let input = vec![make_choice("abcdefghijk"), make_choice("abc")];
        let mut choices = Choices::new(2, &input, false);
        choices.filter(&[]);
        for _ in 0..20 {
            choices.shift_horizontal(true, Some(5));
        }
        assert_eq!(choices.horizontal_offset, 6);
        assert_eq!(choices.current_match().searchable, "abcdefghijk");
        assert_eq!(choices.draw_choices_with_width(Some(5)),
            format!("{}\n\r{}", cursor::nowrap(&color::inverse("ghijk")), cursor::nowrap("")));
        choices.shift_horizontal(false, Some(5));
        assert_eq!(choices.horizontal_offset, 5);
        choices.filter(&['a']);
        assert_eq!(choices.horizontal_offset, 0);
        for _ in 0..3 {
            choices.shift_horizontal(false, Some(5));
        }
        assert_eq!(choices.horizontal_offset, 0);
        choices.filter(&['z']);
        choices.shift_horizontal(true, Some(5));
        assert_eq!(choices.horizontal_offset, 0);
    }

    #[test]
    fn scrolling_uses_visible_rows_and_clamps_on_resize_and_navigation() {
        let input = vec![make_choice("abcdefghijk"), make_choice("xy"), make_choice("yz")];
        let mut choices = Choices::new(2, &input, false);
        choices.filter(&[]);
        for _ in 0..10 {
            choices.shift_horizontal(true, Some(5));
        }
        assert_eq!(choices.horizontal_offset, 6);
        assert_eq!(choices.draw_choices_with_width(Some(20)),
            format!("{}\n\r{}", cursor::nowrap(&color::inverse("abcdefghijk")), cursor::nowrap("xy")));
        assert_eq!(choices.horizontal_offset, 6);
        assert_eq!(choices.draw_choices_with_width(Some(5)),
            format!("{}\n\r{}", cursor::nowrap(&color::inverse("ghijk")), cursor::nowrap("")));
        choices.select_next();
        assert_eq!(choices.draw_choices_with_width(Some(5)),
            format!("{}\n\r{}", cursor::nowrap(&color::inverse("xy")), cursor::nowrap("yz")));
        choices.shift_horizontal(true, Some(5));
        assert_eq!(choices.horizontal_offset, 0);
        choices.previous();
        choices.shift_horizontal(true, None);
        assert_eq!(choices.horizontal_offset, 0);
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
