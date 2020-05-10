use crate::bonus;

pub struct Choice<'a> {
    pub content:  &'a str,
    pub bonus: Vec<f32>,
    pub len: usize,
}

impl<'a> Choice<'a> {
    pub fn new(content: &str) -> Choice {
        let bonus = bonus::compute(&content.chars().collect::<Vec<char>>());
        let len = content.chars().count();

        Choice { content, bonus, len }
    }
}
