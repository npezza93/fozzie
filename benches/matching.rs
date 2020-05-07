#![feature(test)]

extern crate test;

use fozzie::matcher::Match;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_matching(b: &mut test::Bencher) {
        let choice = "Gemfile";
        let query = ['g', 'e', 'm'];

        b.iter(|| Match::is_match(&query, &choice))
    }
}
