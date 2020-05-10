use crate::bonus;
use crate::config::Config;

pub struct Choice<'a> {
    pub searchable:  &'a str,
    pub lower_searchable:  Vec<char>,
    pub searchable_len: usize,
    pub returnable: &'a str,
    pub bonus: Vec<f32>,
}

impl<'a> Choice<'a> {
    pub fn new(content: &'a str, config: &Config) -> Choice<'a> {
        match &config.delimiter {
            Some(delimiter_regex) => {
                let split_content: Vec<&str> = delimiter_regex.split(content).collect();

                let searchable = split_content[config.field.unwrap()];
                let returnable = match config.output {
                    Some(o) => split_content[o],
                    None => searchable
                };

                let (bonus, searchable_len, lower_searchable) = parse_searchable(&searchable);

                Choice { searchable, returnable, bonus, searchable_len, lower_searchable }
            },
            None => {
                let (bonus, searchable_len, lower_searchable) = parse_searchable(&content);

                Choice { searchable: content, returnable: content, bonus, searchable_len, lower_searchable }
            }
        }
    }

}

fn parse_searchable(searchable: &str) -> (Vec<f32>, usize, Vec<char>) {
    let bonus = bonus::compute(&searchable.chars().collect::<Vec<char>>());
    let searchable_len = searchable.chars().count();
    let lower_searchable: Vec<char> = searchable.chars().map(|cchar| cchar.to_ascii_lowercase()).collect();

    (bonus, searchable_len, lower_searchable)
}
