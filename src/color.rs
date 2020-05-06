pub fn inverse(text: &str) -> String {
    format!("\x1B[7m{}\x1B[27m", text)
}

pub fn highlight(c: char) -> String {
    // format!("\x1B[33m{}\x1B[39m", c)
    format!("\x1B[35m{}\x1B[39m", c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse() {
        assert_eq!("\x1B[7mfoo\x1B[27m", inverse("foo"));
    }

    #[test]
    fn test_highlight() {
        // assert_eq!("\x1B[33mf\x1B[39m", highlight('f'));
        assert_eq!("\x1B[35mf\x1B[39m", highlight('f'));
    }
}
