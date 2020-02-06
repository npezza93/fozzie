use std::f64::{INFINITY, NEG_INFINITY};
use crate::bonus;

const MAX: f64 = INFINITY;
pub const MIN: f64 = NEG_INFINITY;
const GAP_TRAILING: f64 = -0.005;
const GAP_INNER: f64 = -0.01;
const GAP_LEADING:       f64 = -0.005;
const MATCH_CONSECUTIVE: f64 = 1.0;

pub fn score(query: &[char], choice: &str) -> (f64, Vec<usize>) {
    let query_length = query.len();
    let choice_length = choice.chars().count();

    // empty needle
    if query_length == 0 {
        return (MIN, vec![]);
    }

    // We only get here if we match so lengths match they
    if query_length == choice_length {
        return (MAX, (0..query_length).collect());
    }

    let bonus = bonus::compute(&choice.chars().collect());
    let mut d = vec![vec![0 as f64; choice_length]; query_length];
    let mut m = vec![vec![0 as f64; choice_length]; query_length];

    query.iter().enumerate().for_each(|(i, qchar)| {
        let mut prev_score = MIN;
        let gap_score = if i == query_length - 1 {
            GAP_TRAILING
        } else {
            GAP_INNER
        };

        choice.chars().enumerate().for_each(|(j, cchar)| {
            if cchar.eq_ignore_ascii_case(qchar) {
                let score = if i == 0 {
                    (j as f64 * GAP_LEADING) + bonus[j]
                } else if j > 0 {
                    let m_score = m[i - 1][j - 1];

                    (m_score + bonus[j]).max(m_score + MATCH_CONSECUTIVE)
                } else {
                    MIN
                };

                prev_score = score.max(prev_score + gap_score);

                d[i][j] = score;
                m[i][j] = prev_score;
            } else {
                prev_score = prev_score + gap_score;

                d[i][j] = MIN;
                m[i][j] = prev_score;
            }
        });
    });

    (m[query_length - 1][choice_length - 1], vec![])
}
