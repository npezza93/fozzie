#![feature(test)]

extern crate test;

use fozzie::matcher::Match;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_normal_scoring(b: &mut test::Bencher) {
        let choice = "CODE_OF_CONDUCT.md";
        let query = ['c', 'o', 'd', 'e'];

        b.iter(|| Match::new(&query, &choice))
    }

    #[bench]
    fn bench_scoring_empty_query(b: &mut test::Bencher) {
        let choice = "CODE_OF_CONDUCT.md";
        let query = [];

        b.iter(|| Match::new(&query, &choice))
    }

    #[bench]
    fn bench_scoring_entire_query(b: &mut test::Bencher) {
        let choice = "gem";
        let query = ['g', 'e', 'm'];

        b.iter(|| Match::new(&query, &choice))
    }
}
