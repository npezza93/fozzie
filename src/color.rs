pub fn inverse(text: &str) -> String {
    format!("\x1B[7m{}\x1B[27m", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse() {
        assert_eq!("\x1B[7mfoo\x1B[27m", inverse("foo"));
    }
}
