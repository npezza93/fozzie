use std::f32::{INFINITY, NEG_INFINITY};
use crate::choice::Choice;
use crate::matrix::Matrix;
use float_cmp::approx_eq;

const MAX: f32 = INFINITY;
pub const MIN: f32 = NEG_INFINITY;
const GAP_TRAILING: f32 = -0.005;
const GAP_INNER: f32 = -0.01;
const GAP_LEADING:       f32 = -0.005;
const MATCH_CONSECUTIVE: f32 = 1.0;

pub struct Score {
    pub score: f32,
    pub positions: Vec<usize>,
}

fn positions(choice_length: usize, query_length: usize, main: Matrix, diagonal: Matrix) -> Vec<usize> {
    let mut positions = vec![0_usize; query_length];

    let mut match_required = false;
    let mut choice_index = choice_length - 1;

    for query_index in (0..query_length).rev() {
        while choice_index > 0_usize {
            let d = diagonal[(query_index, choice_index)];
            let m = main[(query_index, choice_index)];

            if d != MIN && (match_required || approx_eq!(f32, d, m)) {
                // If this score was determined using
                // MATCH_CONSECUTIVE, the previous character MUST be a match
                match_required = query_index > 0 && choice_index > 0 &&
                    approx_eq!(f32, m, diagonal[(query_index - 1, choice_index - 1)] + MATCH_CONSECUTIVE);
                positions[query_index] = choice_index;
                choice_index -= 1;
                break;
            }
            choice_index -= 1;
        }
    }

    positions
}

fn compute(query: &[char], choice: &Choice, query_length: usize, choice_length: usize) -> (Matrix, Matrix){
    let lower_query: Vec<char> = query.iter().map(|qchar| qchar.to_ascii_lowercase()).collect();
    let mut diagonal = Matrix::new(query_length, choice_length);
    let mut main = Matrix::new(query_length, choice_length);

    lower_query.iter().enumerate().for_each(|(i, qchar)| {
        let mut prev_score = MIN;
        let gap_score = if i == query_length - 1 {
            GAP_TRAILING
        } else {
            GAP_INNER
        };

        choice.lower_searchable.iter().enumerate().for_each(|(j, cchar)| {
            if qchar == cchar {
                let bonus_score = choice.bonus[j];

                let current_score = if i == 0 {
                    (j as f32 * GAP_LEADING) + bonus_score
                } else if j > 0 {
                    let m_score = main[(i - 1, j - 1)];
                    let d_score = diagonal[(i - 1, j - 1)];

                    (m_score + bonus_score).max(d_score + MATCH_CONSECUTIVE)
                } else {
                    MIN
                };

                prev_score = current_score.max(prev_score + gap_score);

                diagonal[(i, j)] = current_score;
                main[(i, j)] = prev_score;
            } else {
                prev_score += gap_score;

                diagonal[(i, j)] = MIN;
                main[(i, j)] = prev_score;
            }
        });
    });

    (main, diagonal)
}

impl Score {
    pub fn new(query: &[char], choice: &Choice) -> Score {
        let query_length = query.len();

        if query_length == 0 {
            // empty needle
            Score { score: MIN, positions: vec![] }
        } else if query_length == choice.searchable_len {
            // We only get here if we match so lengths match they
            Score { score: MAX, positions: (0..query_length).collect() }
        } else {
            let (main, diagonal) = compute(query, choice, query_length, choice.searchable_len);

            Score {
                score: main[(query_length - 1, choice.searchable_len - 1)],
                positions: positions(choice.searchable_len, query_length, main, diagonal)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus;
    use crate::config::Config;

    #[test]
    fn prefer_starts_of_words_test() {
        assert!(score("amor", "app/models/order") > score("amor", "app/models/zrder"));
    }

    #[test]
    fn prefer_contiguous_over_letter_following_period_test() {
	assert!(score("gemfil", "Gemfile.lock") < score("gemfil", "Gemfile"));
    }

    #[test]
    fn prefer_shorter_matches_test() {
        assert!(score("abce", "abcdef") > score("abce", "abc de"));
        assert!(score("abc", "    a b c ") > score("abc", " a  b  c "));
        assert!(score("abc", " a b c    ") > score("abc", " a  b  c "));
    }

    #[test]
    fn should_prefer_shorter_candidates_test() {
	assert!(score("test", "tests") > score("test", "testing"));
    }

    #[test]
    fn should_prefer_start_of_candidate_test() {
        assert!(score("test", "testing") > score("test", "/testing"));
    }

    #[test]
    fn score_exact_match_test() {
        assert_eq!(MAX, score("abc", "abc"));
        assert_eq!(MAX, score("aBc", "abC"));
    }

    #[test]
    fn score_empty_query_test() {
        assert_eq!(MIN, score("", ""));
        assert_eq!(MIN, score("", "a"));
        assert_eq!(MIN, score("", "bb"));
    }

    #[test]
    fn score_words_higher_test() {
        assert!(score("orderitem", "app/models/o2/order_item.rb") > score("orderitem", "app/frontend/src/javascript/controllers/OrderItemsWizard.js"));
    }

    #[test]
    fn score_gaps_test() {
        assert_eq!(GAP_LEADING, score("a", "*a"));
        assert_eq!(GAP_LEADING * 2.0, score("a", "*ba"));
        assert_eq!(GAP_LEADING * 2.0 + GAP_TRAILING, score("a", "**a*"));
        assert_eq!(GAP_LEADING * 2.0 + GAP_TRAILING * 2.0, score("a", "**a**"));
        assert_eq!(GAP_LEADING * 2.0 + MATCH_CONSECUTIVE + GAP_TRAILING * 2.0, score("aa", "**aa**"));
        assert_eq!(GAP_LEADING + GAP_LEADING + GAP_INNER + GAP_TRAILING + GAP_TRAILING, score("aa", "**a*a**"));
    }

    #[test]
    fn score_consecutive_test() {
        assert_eq!(GAP_LEADING + MATCH_CONSECUTIVE, score("aa", "*aa"));
        assert_eq!(GAP_LEADING + MATCH_CONSECUTIVE * 2.0, score("aaa", "*aaa"));
        assert_eq!(GAP_LEADING + GAP_INNER + MATCH_CONSECUTIVE, score("aaa", "*a*aa"));
    }

    #[test]
    fn score_slash_test() {
        assert_eq!(GAP_LEADING + bonus::SLASH, score("a", "/a"));
        assert_eq!(GAP_LEADING * 2.0 + bonus::SLASH, score("a", "*/a"));
        assert_eq!(GAP_LEADING * 2.0 + bonus::SLASH + MATCH_CONSECUTIVE, score("aa", "a/aa"));
    }

    #[test]
    fn score_capital_test() {
        assert_eq!(GAP_LEADING + bonus::CAPITAL, score("a", "bA"));
        assert_eq!(GAP_LEADING * 2.0 + bonus::CAPITAL, score("a", "baA"));
        assert_eq!(GAP_LEADING * 2.0 + bonus::CAPITAL + MATCH_CONSECUTIVE, score("aa", "baAa"));
    }

    #[test]
    fn score_dot_test() {
        assert_eq!(GAP_LEADING + bonus::DOT, score("a", ".a"));
        assert_eq!(GAP_LEADING * 3.0 + bonus::DOT, score("a", "*a.a"));
        assert_eq!(GAP_LEADING + GAP_INNER + bonus::DOT, score("a", "*a.a"));
    }

    #[test]
    fn positions_consecutive_test() {
        let positions = positions("amo", "app/models/foo");

        assert_eq!(0, positions[0]);
        assert_eq!(4, positions[1]);
        assert_eq!(5, positions[2]);
        assert_eq!(3, positions.len());
    }

    #[test]
    fn positions_start_of_word_test() {
	let positions = positions("amor", "app/models/order");

	assert_eq!(0, positions[0]);
	assert_eq!(4, positions[1]);
	assert_eq!(11, positions[2]);
	assert_eq!(12, positions[3]);
        assert_eq!(4, positions.len());
    }

    #[test]
    fn positions_no_bonuses_test() {
	let places = positions("as", "tags");

	assert_eq!(1, places[0]);
	assert_eq!(3, places[1]);
	assert_eq!(2, places.len());

	let places = positions("as", "examples.txt");
	assert_eq!(2, places[0]);
	assert_eq!(7, places[1]);
        assert_eq!(2, places.len());
    }

    #[test]
    fn positions_multiple_candidates_start_of_words_test() {
	let positions = positions("abc", "a/a/b/c/c");

	assert_eq!(2, positions[0]);
	assert_eq!(4, positions[1]);
	assert_eq!(6, positions[2]);
        assert_eq!(3, positions.len());
    }

    #[test]
    fn positions_exact_match_test() {
	let positions = positions("foo", "foo");

	assert_eq!(0, positions[0]);
	assert_eq!(1, positions[1]);
	assert_eq!(2, positions[2]);
        assert_eq!(3, positions.len());
    }

    #[test]
    fn positions_test() {
	let positions = positions("code", "CODE_OF_CONDUCT.md");

	assert_eq!(0, positions[0]);
	assert_eq!(1, positions[1]);
	assert_eq!(2, positions[2]);
	assert_eq!(3, positions[3]);
        assert_eq!(4, positions.len());
    }

    #[test]
    fn class_positions_test() {
	let positions = positions("class", "class_");

	assert_eq!(0, positions[0]);
	assert_eq!(1, positions[1]);
	assert_eq!(2, positions[2]);
	assert_eq!(3, positions[3]);
	assert_eq!(4, positions[4]);
        assert_eq!(5, positions.len());
    }

    #[bench]
    fn bench_normal_scoring(b: &mut test::Bencher) {
        let choice = Choice::new("CODE_OF_CONDUCT.md", &config());
        let query = ['c', 'o', 'd', 'e'];

        b.iter(|| compute(&query, &choice, 4, choice.searchable_len))
    }

    #[bench]
    fn bench_scoring_empty_query(b: &mut test::Bencher) {
        let choice = Choice::new("CODE_OF_CONDUCT.md", &config());
        let query = [];

        b.iter(|| Score::new(&query, &choice))
    }

    #[bench]
    fn bench_scoring_entire_query(b: &mut test::Bencher) {
        let choice = Choice::new("gem", &config());
        let query = ['g', 'e', 'm'];

        b.iter(|| Score::new(&query, &choice))
    }

    fn score(choice: &str, query: &str) -> f32 {
        Score::new(
            &choice.chars().collect::<Vec<char>>(),
            &Choice::new(query, &config())
        ).score
    }

    fn positions(choice: &str, query: &str) -> Vec<usize> {
        Score::new(
            &choice.chars().collect::<Vec<char>>(),
            &Choice::new(query, &config())
        ).positions
    }

    fn config() -> Config {
        Config {
            lines: 10, prompt: ">".to_string(), show_scores: false,
            query: None, delimiter: None, field: None, output: None,
            benchmark: false,
        }
    }
}
