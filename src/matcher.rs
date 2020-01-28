use crate::choice::Choice;

pub fn matches(needle: Vec<char>, haystack: &Choice) -> bool {
    needle.is_empty() || needle.iter().all(|nchar| haystack.contains(*nchar))
}
