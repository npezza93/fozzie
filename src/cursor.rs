use termion::clear;

pub fn up(n: usize) -> String {
    format!("\x1B[{}A", n)
}

pub fn down(n: usize) -> String {
    format!("\x1B[{}B", n)
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

pub fn clear_screen_down() -> String {
    "\x1B[J".to_string()
}

pub fn save_position() -> String {
    "\x1B7".to_string()
}

pub fn restore_position() -> String {
    "\x1B8".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_up() {
        assert_eq!("\x1B[4A", up(4));
    }

    #[test]
    fn test_down() {
        assert_eq!("\x1B[4B", down(4));
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
    fn test_clear_screen_down() {
        assert_eq!("\x1B[J", clear_screen_down());
    }

    #[test]
    fn test_save_position() {
        assert_eq!("\x1B7", save_position());
    }

    #[test]
    fn test_restore_position() {
        assert_eq!("\x1B8", restore_position());
    }
}
