use crate::color;

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
}
