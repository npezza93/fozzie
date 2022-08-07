pub const SLASH: f32 = 0.9;
pub const WORD: f32 = 0.87;
pub const DOT: f32 = 0.6;
pub const CAPITAL: f32 = 0.68;
pub const FILENAME: f32 = 0.1;

pub fn compute(choice_chars: &[char]) -> Vec<f32> {
    let mut last_char = '/';
    let last_section = choice_chars.iter().filter(|&n| *n == '/').count();
    let mut section = 0;

    choice_chars
        .iter()
        .map(|&cchar| {
            let mut bonus = for_char(last_char, cchar);
            last_char = cchar;

            if cchar == '/' {
                section += 1;
            } else if section == last_section {
                bonus += FILENAME;
            }

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
    use float_cmp::approx_eq;

    #[test]
    fn test_compute_dir() {
        assert_eq!(
            compute(&"a/b/c/d".chars().collect::<Vec<char>>()),
            vec![0.9, 0.0, 0.9, 0.0, 0.9, 0.0, 1.0]
        );
    }

    #[test]
    fn test_compute_capital() {
        let bonuses = compute(&"aTestString".chars().collect::<Vec<char>>());

        assert_eq!(1.0, bonuses[0]);
        approx_eq!(f32, 0.78, bonuses[1], epsilon = 0.001);
        assert_eq!(0.1, bonuses[2]);
        assert_eq!(0.1, bonuses[3]);
        assert_eq!(0.1, bonuses[4]);
        approx_eq!(f32, 0.78, bonuses[5], epsilon = 0.001);
        assert_eq!(0.1, bonuses[6]);
        assert_eq!(0.1, bonuses[7]);
        assert_eq!(0.1, bonuses[8]);
        assert_eq!(0.1, bonuses[9]);
        assert_eq!(0.1, bonuses[10]);
    }
}
