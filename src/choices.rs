use crate::choice;
use crate::cursor;

pub struct Choices<'a> {
    choices: &'a [String],
    selected: usize,
    max_choices: usize,
}

impl<'a> Choices<'a> {
    const OFFSET: usize = 1;

    pub fn new(lines: usize, choices: &'a [String]) -> Choices<'a> {
        let max_choices = if choices.len() < lines {
            choices.len()
        } else {
            lines
        };

        Choices {
            selected: 0,
            choices,
            max_choices,
        }
    }

    pub fn draw(&self) -> String {
        format!(
            "{}{}\r{}\r{}{}",
            cursor::save_position(),
            cursor::down(),
            cursor::clear_screen_down(),
            self.draw_choices(),
            cursor::restore_position(),
        )
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

    fn last_index(&self) -> usize {
        self.choices.len() - 1
    }

    fn draw_choices(&self) -> String {
        let index = self.starting_position();

        (index..(index + self.max_choices))
            .map(|i| {
                let choice = &self.choices[i];
                format!("{}\n\r", choice::draw(choice, i == self.selected))
            })
            .collect::<Vec<String>>()
            .join("")
    }

    fn starting_position(&self) -> usize {
        if self.selected + Self::OFFSET < self.max_choices {
            0
        } else if self.selected + Self::OFFSET + 1 >= self.choices.len() {
            self.choices.len() - self.max_choices
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
    fn test_draw() {
        let input: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        let choices = Choices::new(4, &input);

        assert_eq!(
            format!(
                "{}{}\r{}\r{}\n\rbar\n\r{}",
                cursor::save_position(),
                cursor::down(),
                cursor::clear_screen_down(),
                color::inverse("foo"),
                cursor::restore_position()
            ),
            choices.draw()
        );
    }

    #[test]
    fn test_previous_when_wrapping() {
        let input = vec!["foo".to_string(), "bar".to_string()];
        let mut choices = Choices::new(4, &input);

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
        choices.selected = 0;

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
        let choices = Choices::new(4, &input);

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
