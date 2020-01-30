pub fn matches(needle: Vec<char>, haystack: &str) -> bool {
    needle.is_empty() || needle.iter().all(|nchar| haystack.contains(*nchar))
}
