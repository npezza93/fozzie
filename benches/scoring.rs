#![feature(test)]

extern crate test;

use fozzie::matcher::Match;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_scoring(b: &mut test::Bencher) {
        let choice = "CODE_OF_CONDUCT.md";
        let query = ['c', 'o', 'd', 'e'];

        b.iter(|| Match::new(&query, &choice))
    }
}
