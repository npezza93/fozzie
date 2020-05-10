use crate::bonus;

pub struct Choice<'a> {
    pub content:  &'a str,
    pub lower_content:  Vec<char>,
    pub bonus: Vec<f32>,
    pub len: usize,
}

impl<'a> Choice<'a> {
    pub fn new(content: &str) -> Choice {
        let bonus = bonus::compute(&content.chars().collect::<Vec<char>>());
        let len = content.chars().count();
        let lower_content = content.chars().map(|cchar| cchar.to_ascii_lowercase()).collect();

        Choice { content, bonus, len, lower_content }
    }
}
