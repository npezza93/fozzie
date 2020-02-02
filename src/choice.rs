use crate::color;

pub fn draw(choice: &str, selected: bool) -> String {
    if selected {
        color::inverse(choice)
    } else {
        choice.into()
    }
}

pub fn contains(choice: &str, character: char) -> bool {
    choice
        .chars()
        .any(|cchar| cchar.eq_ignore_ascii_case(&character))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_not_selected() {
        assert_eq!("foo", draw("foo", false));
    }

    #[test]
    fn test_draw_selected() {
        assert_eq!("\x1B[7mfoo\x1B[27m", draw("foo", true));
    }

    #[test]
    fn test_contains() {
        assert!(contains("foo", 'f'));
        assert!(!contains("foo", 'z'));
    }
}
