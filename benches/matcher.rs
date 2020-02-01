use criterion::{criterion_group, criterion_main, Criterion};

use fozzie::matcher;

fn matches(c: &mut Criterion) {
    c.bench_function("matcher", |b| {
        let choice = "Gemfile";
        let query = ['g', 'e', 'm'];

        b.iter(|| matcher::matches(&query, &choice));
    });
}

criterion_group!(benches, matches);
criterion_main!(benches);
