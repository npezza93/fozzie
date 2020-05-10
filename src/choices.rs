use rayon::prelude::*;

use crate::choice::Choice;
use crate::cursor;
use crate::matcher::Match;
use crate::terminal::Terminal;

pub struct Choices<'a> {
    choices: &'a [Choice],
    selected: usize,
    max_choices: usize,
    matches: Vec<Match<'a>>,
    show_scores: bool,
}

impl<'a> Choices<'a> {
    const OFFSET: usize = 1;

    pub fn new(max_choices: usize, choices: &'a [Choice], show_scores: bool) -> Choices<'a> {
        Choices {
            selected: 0,
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
        println!("{}", self.current_match());
    }

    pub fn current_match(&self) -> &str {
        &self.matches[self.selected].choice.content
    }

    pub fn cancel(&self) -> String {
        format!("\r{}", cursor::clear_screen_down())
    }

    pub fn filter(&mut self, query: &[char]) -> String {
        self.filter_choices(&query);
        self.draw()
    }

    fn filter_choices(&mut self, query: &[char]) {
        self.selected = 0;
        self.matches = self
            .choices
            .par_iter()
            .filter_map(|choice| Match::new(&query, &choice))
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

    fn draw_choices(&self) -> String {
        self.drawn_range()
            .map(|i| self.matches[i].draw(i == self.selected, self.show_scores))
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

    #[test]
    fn test_new() {
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
            Choice::new("baz".to_string()),
            Choice::new("boo".to_string()),
        ];
        let choices = Choices::new(4, &input, false);

        assert_eq!(4, choices.max_choices);
        assert_eq!(0, choices.selected);
        assert_eq!(
            vec!["foo", "bar", "baz", "boo"],
            choices.choices.iter().map(|choice| choice.content.clone()).collect::<Vec<String>>(),
        );
    }

    #[test]
    fn test_new_max_choices() {
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
            Choice::new("baz".to_string()),
            Choice::new("boo".to_string()),
        ];
        let choices = Choices::new(2, &input, false);

        assert_eq!(2, choices.max_choices);
    }

    #[test]
    fn test_filter() {
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
        ];
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
                .map(|matcher| matcher.choice.content.clone())
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_previous_when_wrapping() {
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
        ];
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
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
        ];
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
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
        ];
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
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
        ];
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
        let input: Vec<Choice> = vec![
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
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
            Choice::new("foo".to_string()),
            Choice::new("bar".to_string()),
        ];
        let mut choices = Choices::new(4, &input, false);
        choices.filter(&[]);

        assert_eq!("foo", choices.current_match());
    }

    #[bench]
    fn bench_filtering(b: &mut test::Bencher) {
        let choices: Vec<Choice> = vec![
            Choice::new("CODE_OF_CONDUCT.md".to_string()),
            Choice::new("Cargo.lock".to_string()),
            Choice::new("Cargo.toml".to_string()),
            Choice::new("LICENSE".to_string()),
            Choice::new("README.md".to_string()),
            Choice::new("benches/choices.rs".to_string()),
            Choice::new("benches/drawing.rs".to_string()),
            Choice::new("benches/matching.rs".to_string()),
            Choice::new("benches/scoring.rs".to_string()),
            Choice::new("src/bonus.rs".to_string())
        ];
        let query = ['c', 'o', 'd', 'e'];

        b.iter(|| Choices::new(10, &choices, false).filter(&query))
    }
}
