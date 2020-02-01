use crate::choice;

pub fn matches(needle: &[char], haystack: &str) -> bool {
    needle.is_empty()
        || needle
            .iter()
            .all(|&nchar| choice::contains(&haystack, nchar))
}
