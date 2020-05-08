pub const SLASH: f64 = 0.9;
pub const WORD: f64 = 0.85;
pub const DOT: f64 = 0.6;
pub const CAPITAL: f64 = 0.7;

pub fn compute(choice_chars: &[char]) -> Vec<f64> {
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

fn for_char(prev: char, current: char) -> f64 {
    match current {
        'a'..='z' | '0'..='9' => for_previous(prev),
        'A'..='Z' => match prev {
            'a'..='z' => CAPITAL,
            _ => for_previous(prev),
        },
        _ => 0.0,
    }
}

fn for_previous(ch: char) -> f64 {
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
            vec![0.9, 0.7, 0.0, 0.0, 0.0, 0.7, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
    }
}
