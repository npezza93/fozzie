pub const SLASH: f32 = 0.9;
pub const WORD: f32 = 0.87;
pub const DOT: f32 = 0.6;
pub const CAPITAL: f32 = 0.68;

pub fn compute(choice_chars: &[char]) -> Vec<f32> {
    let mut last_char = '/';

    choice_chars
        .iter()
        .map(|&cchar| {
            let bonus = for_char(last_char, cchar);
            last_char = cchar;
            bonus
        })
        .collect()
}

fn for_char(prev: char, current: char) -> f32 {
    match current {
        'a'..='z' | '0'..='9' => for_previous(prev),
        'A'..='Z' => match prev {
            'a'..='z' => CAPITAL,
            _ => for_previous(prev),
        },
        _ => 0.0,
    }
}

fn for_previous(ch: char) -> f32 {
    match ch {
        '/' => SLASH,
        '-' | '_' | ' ' => WORD,
        '.' => DOT,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute() {
        assert_eq!(
            compute(&"a/b/c/d".chars().collect::<Vec<char>>()),
            vec![0.9, 0.0, 0.9, 0.0, 0.9, 0.0, 0.9]
        );
        assert_eq!(
            compute(&"aTestString".chars().collect::<Vec<char>>()),
            vec![0.9, 0.68, 0.0, 0.0, 0.0, 0.68, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
    }
}
