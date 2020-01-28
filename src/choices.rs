use crate::choice::Choice;
use crate::cursor;
use std::io::{BufRead, Write};

pub struct Choices<W> {
    choices: Vec<Choice>,
    selected: usize,
    max_choices: usize,
    output: W,
}

impl<W: Write> Choices<W> {
    const OFFSET: usize = 1;

    pub fn new<R: BufRead>(lines: usize, output: W, input: R) -> Choices<W> {
        let choices: Vec<Choice> = input
            .lines()
            .map(|line| line.unwrap())
            .map(Choice::new)
            .collect();

        let max_choices = if choices.len() <= lines {
            choices.len()
        } else {
            lines
        };

        Choices {
            selected: 0,
            output: output,
            choices,
            max_choices,
        }
    }

    pub fn draw(&mut self) {
        self.print(format!(
            "{}{}\r{}\r{}{}",
            cursor::save_position(),
            cursor::down(1),
            cursor::clear_screen_down(),
            self.draw_choices(),
            cursor::restore_position(),
        ));
    }

    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.last_index();
        } else {
            self.selected -= 1;
        }

        self.draw();
    }

    pub fn next(&mut self) {
        if self.selected == self.last_index() {
            self.selected = 0;
        } else {
            self.selected += 1;
        }

        self.draw();
    }

    pub fn select(&mut self) {
        self.print(format!(
            "{}{}{}",
            cursor::col(0),
            cursor::clear_screen_down(),
            self.choices[self.selected].content
        ));
    }

    pub fn select_none(&mut self) {
        self.print(format!("{}{}", cursor::col(0), cursor::clear_screen_down()));
    }

    fn last_index(&self) -> usize {
        self.choices.len() - 1
    }

    fn draw_choices(&self) -> String {
        let index = self.starting_position();

        (index..(index + self.max_choices))
            .map(|i| {
                let choice = &self.choices[i];
                format!("{}\n\r", choice.draw(i == self.selected))
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

    fn print(&mut self, text: String) {
        self.output.write(text.as_bytes()).unwrap();
        self.output.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;
    use std::io::Cursor;

    #[test]
    fn test_new() {
        let mut output = Cursor::new(vec![]);
        let input = "foo\nbar\nbaz\nboo\n".as_bytes();
        let choices = Choices::new(4, &mut output, input);

        let mut expected_choices: Vec<Choice> = Vec::new();
        expected_choices.push(Choice::new(String::from("foo")));
        expected_choices.push(Choice::new(String::from("bar")));
        expected_choices.push(Choice::new(String::from("baz")));
        expected_choices.push(Choice::new(String::from("boo")));

        assert_eq!(4, choices.max_choices);
        assert_eq!(0, choices.selected);
        assert_eq!(expected_choices, choices.choices);
    }

    #[test]
    fn test_draw() {
        let mut cursor = Cursor::new(vec![]);
        let input = "foo\nbar\n".as_bytes();
        let mut choices = Choices::new(4, &mut cursor, input);

        choices.draw();
        let actual = cursor.into_inner();

        assert_eq!(
            format!(
                "{}{}\r{}\r{}\n\rbar\n\r{}",
                cursor::save_position(),
                cursor::down(1),
                cursor::clear_screen_down(),
                color::inverse("foo"),
                cursor::restore_position()
            ),
            String::from_utf8(actual).unwrap()
        );
    }
}
