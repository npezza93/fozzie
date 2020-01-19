use crate::cursor;
use std::io::{stdin, BufRead, Write};

pub struct Choices<W> {
    choices: Vec<String>,
    // selected: u32,
    lines: usize,
    output: W,
}

impl<W: Write> Choices<W> {
    pub fn new(lines: usize, output: W) -> Choices<W> {
        Choices {
            choices: stdin().lock().lines().map(|line| line.unwrap()).collect(),
            // selected: 0,
            lines,
            output,
        }
    }

    fn print(&mut self, text: String) {
        write!(self.output, "{}", text).unwrap();
        self.output.flush().unwrap();
    }

    pub fn render(&mut self) {
        let mut choices = format!("{}\n\r", cursor::move_screen_up(self.lines));
        let choice_count = [self.choices.len(), self.lines];

        if let Some(count) = choice_count.iter().min() {
            self.choices[0..(count - 1)].iter().for_each(|line| {
                choices.push_str(&format!("{}\n\r", line).to_string());
            });

            choices.push_str(&cursor::up(*count));
            self.print(choices);
        };
    }
}
