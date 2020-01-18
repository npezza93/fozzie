use termion::clear;

pub fn up(n: usize) -> String {
    format!("\x1B[{}A", n)
}

pub fn left(n: usize) -> String {
    format!("\x1B[{}D", n)
}

pub fn right(n: usize) -> String {
    format!("\x1B[{}C", n)
}

pub fn clear_char(n: usize) -> String {
    format!("\x1B[{}X", n)
}

pub fn clear_line() -> String {
    format!("{}", clear::CurrentLine)
}

pub fn col(n: usize) -> String {
    format!("\x1B[{}G", n)
}

pub fn move_screen_up(n: usize) -> String {
    format!("{}{}", "\n".repeat(n), up(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_up() {
        assert_eq!("\x1B[4A", up(4));
    }

    #[test]
    fn test_left() {
        assert_eq!("\x1B[5D", left(5));
    }

    #[test]
    fn test_right() {
        assert_eq!("\x1B[6C", right(6));
    }

    #[test]
    fn test_clear_char() {
        assert_eq!("\x1B[7X", clear_char(7));
    }

    #[test]
    fn test_clear_line() {
        assert_eq!("\x1B[2K", clear_line());
    }

    #[test]
    fn test_col() {
        assert_eq!("\x1B[8G", col(8));
    }

    #[test]
    fn test_move_screen_up() {
        assert_eq!("\n\n\n\n\n\x1B[5A", move_screen_up(5));
    }
}
