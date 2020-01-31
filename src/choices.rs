use crate::choice;
use crate::cursor;
use crate::matcher;

pub struct Choices<'a> {
    choices: &'a [String],
    selected: usize,
    max_choices: usize,
    matches: Vec<&'a String>,
}

impl<'a> Choices<'a> {
    const OFFSET: usize = 1;

    pub fn new(max_choices: usize, choices: &'a [String]) -> Choices<'a> {
        Choices {
            selected: 0,
            matches: vec![],
            choices,
            max_choices,
        }
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

    pub fn select(&self) -> String {
        format!(
            "\r{}{}",
            cursor::clear_screen_down(),
            self.choices[self.selected]
        )
    }

    pub fn cancel(&self) -> String {
        format!("\r{}", cursor::clear_screen_down())
    }

    pub fn filter(&mut self, query: &Vec<char>) -> String {
        self.selected = 0;
        self.matches = self
            .choices
            .iter()
            .filter(|choice| matcher::matches(&query, &choice))
            .collect();

        self.draw()
    }

    fn draw(&self) -> String {
        format!(
            "{}{}\r{}\r{}{}",
            cursor::save_position(),
            cursor::down(),
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
            .map(|i| {
                let choice = &self.matches[i];
                format!("{}\n\r", choice::draw(choice, i == self.selected))
            })
            .collect::<Vec<String>>()
            .join("")
    }

    fn drawn_range(&self) -> std::ops::Range<usize> {
        let index = self.starting_position();

        let max_choices = if self.matches.len() < self.max_choices {
            self.matches.len()
        } else {
            self.max_choices
        };

        index..(index + max_choices)
    }

    fn starting_position(&self) -> usize {
        if self.selected + Self::OFFSET < self.max_choices {
            0
        } else if self.selected + Self::OFFSET + 1 >= self.matches.len() && !self.matches.is_empty()
        {
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

    #[test]
    fn test_new() {
        let input: Vec<String> = vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string(),
            "boo".to_string(),
        ];
        let choices = Choices::new(4, &input);

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
        let choices = Choices::new(2, &input);

        assert_eq!(2, choices.max_choices);
    }

    #[test]
    fn test_filter() {
        let input: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input);

        assert_eq!(
            format!(
                "{}{}\r{}\r{}\n\rbar\n\r{}",
                cursor::save_position(),
                cursor::down(),
                cursor::clear_screen_down(),
                color::inverse("foo"),
                cursor::restore_position()
            ),
            choices.filter(&vec![])
        );
        assert_eq!(vec!["foo", "bar"], choices.matches);
    }

    #[test]
    fn test_previous_when_wrapping() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input);
        choices.filter(&vec![]);

        assert_eq!(
            format!(
                "{}{}\r{}\rfoo\n\r{}\n\r{}",
                cursor::save_position(),
                cursor::down(),
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
        let mut choices = Choices::new(4, &input);
        choices.filter(&vec![]);
        choices.selected = 1;

        assert_eq!(
            format!(
                "{}{}\r{}\r{}\n\rbar\n\r{}",
                cursor::save_position(),
                cursor::down(),
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
        let mut choices = Choices::new(4, &input);
        choices.filter(&vec![]);
        choices.selected = 1;

        assert_eq!(
            format!(
                "{}{}\r{}\r{}\n\rbar\n\r{}",
                cursor::save_position(),
                cursor::down(),
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
        let mut choices = Choices::new(4, &input);
        choices.filter(&vec![]);

        assert_eq!(
            format!(
                "{}{}\r{}\rfoo\n\r{}\n\r{}",
                cursor::save_position(),
                cursor::down(),
                cursor::clear_screen_down(),
                color::inverse("bar"),
                cursor::restore_position(),
            ),
            choices.next()
        );
        assert_eq!(1, choices.selected);
    }

    #[test]
    fn test_select() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input);
        choices.filter(&vec![]);

        assert_eq!(
            format!("\r{}foo", cursor::clear_screen_down()),
            choices.select()
        );
    }

    #[test]
    fn test_cancel() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let choices = Choices::new(4, &input);

        assert_eq!(
            format!("\r{}", cursor::clear_screen_down()),
            choices.cancel()
        );
    }
}
