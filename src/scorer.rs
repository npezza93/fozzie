use std::f64::{INFINITY, NEG_INFINITY};
use crate::bonus;

const MAX: f64 = INFINITY;
pub const MIN: f64 = NEG_INFINITY;
const GAP_TRAILING: f64 = -0.005;
const GAP_INNER: f64 = -0.01;
const GAP_LEADING:       f64 = -0.005;
const MATCH_CONSECUTIVE: f64 = 1.0;

pub struct Score {
    query_length: usize,
    choice_length: usize,
    main: Option<Vec<Vec<f64>>>,
    diagonal: Option<Vec<Vec<f64>>>,
    score: Option<f64>,
    positions: Option<Vec<usize>>,
}

impl Score {
    pub fn new(query: &[char], choice: &str) -> Score {
        let mut score = Score {
            query_length: query.len(),
            choice_length: choice.chars().count(),
            main: None,
            diagonal: None,
            score: None,
            positions: None,
        };

        if score.query_length == 0 {
            // empty needle
            score.score = Some(MIN);
            score.positions = Some(vec![]);
        } else if score.query_length == score.choice_length {
            // We only get here if we match so lengths match they
            score.score = Some(MAX);
            score.positions = Some((0..score.query_length).collect());
        } else {
            let bonus = bonus::compute(&choice.chars().collect());
            let mut diagonal = vec![vec![0 as f64; score.choice_length]; score.query_length];
            let mut main = vec![vec![0 as f64; score.choice_length]; score.query_length];

            query.iter().enumerate().for_each(|(i, qchar)| {
                let mut prev_score = MIN;
                let gap_score = if i == score.query_length - 1 {
                    GAP_TRAILING
                } else {
                    GAP_INNER
                };

                choice.chars().enumerate().for_each(|(j, cchar)| {
                    if cchar.eq_ignore_ascii_case(&qchar) {
                        let bonus_score = bonus[j];

                        let current_score = if i == 0 {
                            (j as f64 * GAP_LEADING) + bonus_score
                        } else if j > 0 {
                            let m_score = main[i - 1][j - 1];
                            let d_score = diagonal[i - 1][j - 1];

                            (m_score + bonus_score).max(d_score + MATCH_CONSECUTIVE)
                        } else {
                            MIN
                        };

                        prev_score = current_score.max(prev_score + gap_score);

                        diagonal[i][j] = current_score;
                        main[i][j] = prev_score;
                    } else {
                        prev_score += gap_score;

                        diagonal[i][j] = MIN;
                        main[i][j] = prev_score;
                    }
                });
            });
            score.diagonal = Some(diagonal);
            score.main = Some(main);
        }

        score
    }

    pub fn score(&self) -> f64 {
        match self.score {
            Some(score) => score,
            None => {
                match &self.main {
                    Some(main) => main[self.query_length - 1][self.choice_length - 1],
                    None => MIN,
                }
            }
        }
    }

    pub fn positions(&self) -> Vec<usize> {
        match &self.positions {
            Some(positions) => positions.to_vec(),
            None => {
                let mut positions = vec![0 as usize; self.query_length];

                let mut match_required = false;
                let mut j = self.choice_length - 1;
                let diagonal = self.diagonal.as_ref().unwrap();
                let main = self.main.as_ref().unwrap();

                for i in (0..self.query_length).rev() {
                    while j > (0 as usize) {
                        let last = if i > 0 && j > 0 { diagonal[i - 1][j - 1] } else { 0.0 };

                        let d = diagonal[i][j];
                        let m = main[i][j];

                        if d != MIN && (match_required || d == m) {
                            if i > 0 && j > 0 && m == last + MATCH_CONSECUTIVE {
                                match_required = true;
                            }

                            positions[i] = j;

                            break;
                        }

                        j -= 1
                    }
                }

                positions
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn score(choice: &str, query: &str) -> f64 {
        Score::new(&choice.chars().collect::<Vec<char>>(), query).score()
    }

    fn positions(choice: &str, query: &str) -> Vec<usize> {
        Score::new(&choice.chars().collect::<Vec<char>>(), query).positions()
    }
}
