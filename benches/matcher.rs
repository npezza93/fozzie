use criterion::{criterion_group, criterion_main, Criterion};

use fozzie::matcher::Match;

fn matches(c: &mut Criterion) {
    c.bench_function("matcher", |b| {
        let choice = "Gemfile";
        let query = ['g', 'e', 'm'];

        b.iter(|| Match::is_match(&query, &choice));
    });
}

criterion_group!(benches, matches);
criterion_main!(benches);
