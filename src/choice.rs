use crate::color;

#[derive(Debug, PartialEq)]
pub struct Choice {
    pub content: String,
}

impl Choice {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    pub fn draw(&self, selected: bool) -> String {
        if selected {
            color::inverse(&self.content)
        } else {
            self.content.to_string()
        }
    }

    pub fn contains(&self, character: char) -> bool {
        self.content
            .chars()
            .any(|cchar| cchar.eq_ignore_ascii_case(&character))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let choice = Choice::new("foo".to_string());

        assert_eq!("foo", choice.content);
    }

    #[test]
    fn test_draw_not_selected() {
        let choice = Choice::new("foo".to_string());

        assert_eq!("foo", choice.draw(false));
    }

    #[test]
    fn test_draw_selected() {
        let choice = Choice::new("foo".to_string());

        assert_eq!("\x1B[7mfoo\x1B[0m", choice.draw(true));
    }

    #[test]
    fn test_equality() {
        let choice1 = Choice::new("foo".to_string());
        let choice2 = Choice::new("foo".to_string());
        let choice3 = Choice::new("bar".to_string());

        assert_eq!(choice1, choice2);
        assert_ne!(choice1, choice3);
    }

    #[test]
    fn test_contains() {
        let choice = Choice::new(String::from("foo"));

        assert!(choice.contains('f'));
        assert!(!choice.contains('z'));
    }
}
