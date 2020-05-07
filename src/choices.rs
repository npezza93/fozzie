use crate::cursor;
use crate::matcher::Match;
use crate::terminal::Terminal;

pub struct Choices<'a> {
    choices: &'a [String],
    start_selected: usize,
    end_selected: usize,
    max_choices: usize,
    matches: Vec<Match<'a>>,
    show_scores: bool,
}

impl<'a> Choices<'a> {
    const OFFSET: usize = 1;

    pub fn new(max_choices: usize, choices: &'a [String], show_scores: bool) -> Choices<'a> {
        Choices {
            start_selected: 0,
            end_selected: 0,
            matches: vec![],
            choices,
            max_choices,
            show_scores,
        }
    }

    pub fn initial_draw(&mut self, terminal: &mut Terminal) {
        self.filter_choices(&[]);

        terminal.print(&format!(
            "{}{}\r",
            self.draw_choices(),
            cursor::up(self.max_choices())
        ));
    }

    pub fn previous(&mut self, multiple: bool) -> String {
        if self.start_selected == 0 {
            self.start_selected = self.last_index();
            self.end_selected = self.last_index();
        } else {
            self.end_selected -= 1;
            if !multiple {
                self.start_selected -= 1;
            }
        }

        self.draw()
    }

    pub fn next(&mut self, multiple: bool) -> String {
        if self.start_selected == self.last_index() {
            self.start_selected = 0;
            self.end_selected = 0;
        } else {
            self.end_selected += 1;
            if !multiple {
                self.start_selected += 1;
            }
        }

        self.draw()
    }

    pub fn select(&self, terminal: &mut Terminal) {
        terminal.print(&format!("\r{}", cursor::clear_screen_down()));
        println!("{}", self.matches[self.start_selected]);
    }

    pub fn cancel(&self) -> String {
        format!("\r{}", cursor::clear_screen_down())
    }

    pub fn filter(&mut self, query: &[char]) -> String {
        self.filter_choices(&query);
        self.draw()
    }

    fn filter_choices(&mut self, query: &[char]) {
        self.start_selected = 0;
        self.end_selected = 0;
        self.matches = self
            .choices
            .iter()
            .filter_map(|choice| Match::new(&query, &choice))
            .collect::<Vec<Match<'a>>>();
        self.matches.sort();
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

    fn draw_choices(&self) -> String {
        self.drawn_range()
            .map(|i| self.matches[i].draw((self.start_selected..=self.end_selected).contains(&i), self.show_scores))
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
        if self.start_selected + Self::OFFSET < self.max_choices {
            0
        } else if self.start_selected + Self::OFFSET + 1 >= self.matches.len() {
            self.matches.len() - self.max_choices
        } else {
            self.start_selected + Self::OFFSET + 1 - self.max_choices
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;

    #[test]
    fn test_new() {
        let input: Vec<String> = vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string(),
            "boo".to_string(),
        ];
        let choices = Choices::new(4, &input, false);

        let expected_choices: &[String] = &input;

        assert_eq!(4, choices.max_choices);
        assert_eq!(0, choices.selected);
        assert_eq!(expected_choices, choices.choices);
    }

    #[test]
    fn test_new_max_choices() {
        let input: Vec<String> = vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string(),
            "boo".to_string(),
        ];
        let choices = Choices::new(2, &input, false);

        assert_eq!(2, choices.max_choices);
    }

    #[test]
    fn test_filter() {
        let input: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input, false);

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\rbar{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                color::inverse("foo"),
                cursor::restore_position()
            ),
            choices.filter(&[])
        );
        assert_eq!(
            vec!["foo", "bar"],
            choices
                .matches
                .iter()
                .map(|matcher| matcher.choice)
                .collect::<Vec<&str>>()
        );
    }

    #[test]
    fn test_previous_when_wrapping() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);

        assert_eq!(
            format!(
                "{}\r\n{}\rfoo\n\r{}{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                color::inverse("bar"),
                cursor::restore_position(),
            ),
            choices.previous()
        );
        assert_eq!(1, choices.selected);
    }

    #[test]
    fn test_previous() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);
        choices.selected = 1;

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\rbar{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                color::inverse("foo"),
                cursor::restore_position(),
            ),
            choices.previous()
        );
        assert_eq!(0, choices.selected);
    }

    #[test]
    fn test_next_when_wrapping() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);
        choices.selected = 1;

        assert_eq!(
            format!(
                "{}\r\n{}\r{}\n\rbar{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                color::inverse("foo"),
                cursor::restore_position(),
            ),
            choices.next()
        );
        assert_eq!(0, choices.selected);
    }

    #[test]
    fn test_next() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);

        assert_eq!(
            format!(
                "{}\r\n{}\rfoo\n\r{}{}",
                cursor::save_position(),
                cursor::clear_screen_down(),
                color::inverse("bar"),
                cursor::restore_position(),
            ),
            choices.next()
        );
        assert_eq!(1, choices.selected);
    }

    #[test]
    fn test_cancel() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let choices = Choices::new(4, &input, false);

        assert_eq!(
            format!("\r{}", cursor::clear_screen_down()),
            choices.cancel()
        );
    }
}
