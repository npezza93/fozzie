use criterion::{criterion_group, criterion_main, Criterion};

use fozzie::choice::Choice;
use fozzie::matcher;

fn matches(c: &mut Criterion) {
    c.bench_function("matcher", |b| {
        let choice = Choice::new(String::from("Gemfile"));

        b.iter(|| matcher::matches(vec!['g', 'e', 'm'], &choice));
    });
}

criterion_group!(benches, matches);
criterion_main!(benches);
