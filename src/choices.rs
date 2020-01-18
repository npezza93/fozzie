use crate::cursor;
use crate::writer;
use std::io::{stdin, BufRead};

pub struct Choices {
    choices: Vec<String>,
    // selected: u32,
    lines: usize,
}

impl Choices {
    pub fn new(lines: usize) -> Choices {
        Choices {
            choices: stdin().lock().lines().map(|line| line.unwrap()).collect(),
            // selected: 0,
            lines,
        }
    }

    pub fn render(&self) {
        let mut choices = format!("{}\n\r", cursor::move_screen_up(self.lines));
        let choice_count = [self.choices.len(), self.lines];

        if let Some(count) = choice_count.iter().min() {
            self.choices[0..(count - 1)].iter().for_each(|line| {
                choices.push_str(&format!("{}\n\r", line).to_string());
            });

            choices.push_str(&cursor::up(*count));
            writer::print(choices);
        };
    }
}
