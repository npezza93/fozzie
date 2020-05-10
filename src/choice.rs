use crate::bonus;

pub struct Choice {
    pub content:  String,
    pub bonus: Vec<f32>,
}

impl Choice {
    pub fn new(content: String) -> Choice {
        let bonus = bonus::compute(&content.chars().collect::<Vec<char>>());

        Choice { content, bonus }
    }
}
