use crate::choice;

pub fn matches(needle: &[char], haystack: &str) -> bool {
    needle.is_empty()
        || needle
            .iter()
            .all(|&nchar| choice::contains(&haystack, nchar))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(matches(&['a'], "a"));
        assert!(matches(&['a'], "abc"));
        assert!(matches(&['a', 'b', 'c'], "abc"));
        assert!(matches(&['A', 'B', 'C'], "abc"));
        assert!(matches(&['a', 'b', 'c'], "a1b2c3"));
        assert!(matches(&['a', 'b', 'c'], "a1b2c3"));
        assert!(matches(&['t', 'e', 's', 't'], "t/e/s/t"));
        assert!(matches(&['t', 'e', 's', 't'], "t💣e💣s💣t"));
        assert!(matches(&['💣', '💣', '💣'], "t💣e💣s💣t"));

        assert!(!matches(&['a', 'b', 'c'], "ab"));
        assert!(!matches(&['a', 'b', 'c'], ""));

        assert!(matches(&[], ""));
        assert!(matches(&[], "ab"));

        // UTF-8 case testing
        assert!(matches(&['a'], "A"));
        assert!(matches(&['A'], "a"));
    }
}
