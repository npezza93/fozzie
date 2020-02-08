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
                    if cchar.eq_ignore_ascii_case(qchar) {
                        let current_score = if i == 0 {
                            (j as f64 * GAP_LEADING) + bonus[j]
                        } else if j > 0 {
                            let m_score = main[i - 1][j - 1];

                            (m_score + bonus[j]).max(m_score + MATCH_CONSECUTIVE)
                        } else {
                            MIN
                        };

                        prev_score = current_score.max(prev_score + gap_score);

                        diagonal[i][j] = current_score;
                        main[i][j] = prev_score;
                    } else {
                        prev_score = prev_score + gap_score;

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
            None => self.main.as_ref().unwrap()[self.query_length - 1][self.choice_length - 1]
        }
    }
}
