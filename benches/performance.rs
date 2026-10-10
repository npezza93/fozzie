use criterion::{criterion_group, criterion_main, Criterion};
use fozzie::{choice::Choice, choices::Choices, config::Config, matcher::Match, scorer::Score};
use std::hint::black_box;

fn config() -> Config {
    Config {
        lines: 10,
        prompt: ">".to_string(),
        show_scores: false,
        query: None,
        delimiter: None,
        field: None,
        output: None,
        benchmark: false,
    }
}

fn performance(c: &mut Criterion) {
    let config = config();
    let gemfile = Choice::new("Gemfile", &config);
    c.bench_function("matching", |b| {
        b.iter(|| Match::is_match(black_box(&['g', 'e', 'm']), black_box(&gemfile)))
    });
    c.bench_function("matching_uppercase", |b| {
        b.iter(|| Match::is_match(black_box(&['G', 'e', 'm']), black_box(&gemfile)))
    });

    let choice = Choice::new("CODE_OF_CONDUCT.md", &config);
    let query = ['c', 'o', 'd', 'e'];
    let matched = Match::new(&query, &choice).unwrap();
    c.bench_function("drawing", |b| {
        b.iter(|| black_box(&matched).draw(false, false))
    });
    c.bench_function("normal_scoring", |b| {
        b.iter(|| Score::new(black_box(&query), black_box(&choice)))
    });
    c.bench_function("scoring_empty_query", |b| {
        b.iter(|| Score::new(black_box(&[]), black_box(&choice)))
    });
    let exact = Choice::new("gem", &config);
    c.bench_function("scoring_entire_query", |b| {
        b.iter(|| Score::new(black_box(&['g', 'e', 'm']), black_box(&exact)))
    });

    let choices: Vec<_> = [
        "CODE_OF_CONDUCT.md",
        "Cargo.lock",
        "Cargo.toml",
        "LICENSE",
        "README.md",
        "benches/choices.rs",
        "benches/drawing.rs",
        "benches/matching.rs",
        "benches/scoring.rs",
        "src/bonus.rs",
    ]
    .into_iter()
    .map(|input| Choice::new(input, &config))
    .collect();
    c.bench_function("filtering", |b| {
        b.iter(|| Choices::new(10, black_box(&choices), false).filter(black_box(&query)))
    });
}

criterion_group!(benches, performance);
criterion_main!(benches);
