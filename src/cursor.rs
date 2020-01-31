pub fn col(n: usize) -> String {
    format!("\x1B[{}G", n)
}

pub fn up(n: usize) -> String {
    format!("\x1B[{}A", n)
}

pub fn left() -> &'static str {
    "\x1B[1D"
}

pub fn right() -> &'static str {
    "\x1B[1C"
}

pub fn clear_line() -> &'static str {
    "\x1B[2K"
}

pub fn clear_screen_down() -> &'static str {
    "\x1B[J"
}

pub fn save_position() -> &'static str {
    "\x1B7"
}

pub fn restore_position() -> &'static str {
    "\x1B8"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_up() {
        assert_eq!("\x1B[2A", up(2));
    }

    #[test]
    fn test_left() {
        assert_eq!("\x1B[1D", left());
    }

    #[test]
    fn test_right() {
        assert_eq!("\x1B[1C", right());
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
