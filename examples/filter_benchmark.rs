//! Compare the score-only pipeline with the original eager filtering pipeline.
//! Run `RAYON_NUM_THREADS=4 cargo +nightly-2024-01-01 run --locked --release --example filter_benchmark`.
//! Optional arguments: candidate count (default 100000), path padding (default 0).
use fozzie::{choice::Choice, choices::Choices, config::Config, cursor, matcher::Match};
use rayon::prelude::*;
use std::{hint::black_box, time::Instant};

// Reference implementation of Choices::filter before score-only ranking.
struct Eager<'a> {
    choices: &'a [Choice<'a>],
    matches: Vec<Match<'a>>,
}

impl Eager<'_> {
    fn filter(&mut self, query: &[char]) -> String {
        self.matches = self
            .choices
            .par_iter()
            .filter_map(|choice| Match::new(query, choice))
            .collect();
        self.matches.par_sort();
        let rows = self
            .matches
            .iter()
            .take(10)
            .enumerate()
            .map(|(i, matched)| matched.draw(i == 0, false))
            .collect::<Vec<_>>()
            .join("\n\r");
        format!(
            "{}\r\n{}\r{}{}",
            cursor::save_position(),
            cursor::clear_screen_down(),
            rows,
            cursor::restore_position()
        )
    }
}

fn timed(mut f: impl FnMut() -> String) -> f64 {
    let start = Instant::now();
    black_box(f());
    start.elapsed().as_secs_f64() * 1_000.0
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: usize = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(100_000);
    let padding: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(0);
    let config = Config {
        lines: 10,
        prompt: ">".into(),
        show_scores: false,
        query: None,
        delimiter: None,
        field: None,
        output: None,
        benchmark: false,
    };
    let names = [
        "application_controller.rs",
        "search_index.rs",
        "user_account_test.rs",
        "config_settings.toml",
        "README.md",
        "matrix_compute.rs",
        "performance_test.rs",
        "order_item.rb",
    ];
    let strings: Vec<_> = (0..count)
        .map(|i| {
            format!(
                "project_{:03}/src/{}/module_{:05}/{}",
                i % 37,
                "segment/".repeat(padding / 8),
                i,
                names[i % names.len()]
            )
        })
        .collect();
    let choices: Vec<_> = strings.iter().map(|s| Choice::new(s, &config)).collect();
    let mut eager = Eager {
        choices: &choices,
        matches: vec![],
    };
    let mut optimized = Choices::new(10, &choices, false);

    for text in ["", "src", "appcont", "test", "applicationcontroller", "xyz"] {
        let query: Vec<char> = text.chars().collect();
        assert_eq!(eager.filter(&query), optimized.filter(&query));
        for _ in 0..5 {
            black_box(eager.filter(&query));
            black_box(optimized.filter(&query));
        }
        let mut before = vec![];
        let mut after = vec![];
        let mut ratios = vec![];
        for trial in 0..21 {
            let (a, b) = if trial % 2 == 0 {
                (
                    timed(|| eager.filter(&query)),
                    timed(|| optimized.filter(&query)),
                )
            } else {
                let b = timed(|| optimized.filter(&query));
                let a = timed(|| eager.filter(&query));
                (a, b)
            };
            before.push(a);
            after.push(b);
            ratios.push(a / b);
        }
        before.sort_by(f64::total_cmp);
        after.sort_by(f64::total_cmp);
        ratios.sort_by(f64::total_cmp);
        println!("count={count} padding={padding} query={text:?} eager_ms={:.3} optimized_ms={:.3} paired_speedup={:.2}x", before[10], after[10], ratios[10]);
    }
}
