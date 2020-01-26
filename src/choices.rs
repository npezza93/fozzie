use crate::choice::Choice;
use crate::cursor;
use std::io::{stdin, BufRead, Write};

pub struct Choices<W> {
    choices: Vec<Choice>,
    selected: usize,
    max_choices: usize,
    output: W,
}

impl<W: Write> Choices<W> {
    const OFFSET: usize = 1;

    pub fn new(lines: usize, output: W) -> Choices<W> {
        let choices: Vec<Choice> = stdin()
            .lock()
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
            choices,
            selected: 0,
            output,
            max_choices,
        }
    }

    pub fn inital_draw(&mut self) {
        self.print(format!(
            "{}{}{}",
            cursor::save_position(),
            cursor::move_screen_up(self.max_choices + 1),
            cursor::restore_position(),
        ));
        self.draw_all();
    }

    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.last_index();
        } else {
            self.selected -= 1;
        }

        if self.should_redraw_previous() {
            self.draw_all();
        } else {
            self.swap_active_choices(
                self.previous_down_movement(),
                cursor::up(1),
                self.selected + 1,
            );
        }
    }

    pub fn next(&mut self) {
        if self.selected == self.last_index() {
            self.selected = 0;
        } else {
            self.selected += 1;
        }

        if self.should_redraw_next() {
            self.draw_all();
        } else {
            self.swap_active_choices(
                self.next_down_movement(),
                cursor::down(1),
                self.selected - 1,
            );
        }
    }

    fn should_redraw_next(&self) -> bool {
        self.selected == 0
            || ((self.max_choices - Self::OFFSET) <= self.selected
                && (self.last_index() - Self::OFFSET) >= self.selected)
    }

    fn should_redraw_previous(&self) -> bool {
        self.selected == self.last_index()
            || ((self.max_choices - Self::OFFSET - 1) <= self.selected
                && (self.last_index() - Self::OFFSET - 1) >= self.selected)
    }

    fn next_down_movement(&self) -> usize {
        if (self.last_index() - Self::OFFSET) <= self.selected {
            self.max_choices - Self::OFFSET - 1
                + (Self::OFFSET - (self.last_index() - self.selected))
        } else {
            self.selected % self.max_choices
        }
    }

    fn previous_down_movement(&self) -> usize {
        if (self.last_index() - Self::OFFSET - 1) <= self.selected {
            self.max_choices - Self::OFFSET
                + 1
                + (Self::OFFSET - (self.last_index() - self.selected))
        } else {
            (self.selected + 2) % self.max_choices
        }
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

    fn draw_all(&mut self) {
        self.print(format!(
            "{}{}\r{}\r{}{}",
            cursor::save_position(),
            cursor::down(1),
            cursor::clear_screen_down(),
            self.draw_choices(),
            cursor::restore_position(),
        ));
    }

    fn print(&mut self, text: String) {
        write!(self.output, "{}", text).unwrap();
        self.output.flush().unwrap();
    }

    fn swap_active_choices(
        &mut self,
        initial_down_n: usize,
        movement: String,
        inactive_choice_index: usize,
    ) {
        self.print(format!(
            "{}{}\r{}{}{}\r{}{}{}",
            cursor::save_position(),
            cursor::down(initial_down_n),
            cursor::clear_line(),
            self.choices[inactive_choice_index].draw(false),
            movement,
            cursor::clear_line(),
            self.choices[self.selected].draw(true),
            cursor::restore_position()
        ));
    }
}
