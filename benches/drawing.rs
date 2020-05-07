#![feature(test)]

extern crate test;

use fozzie::matcher::Match;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_drawing(b: &mut test::Bencher) {
        let choice = "CODE_OF_CONDUCT.md";
        let query = ['c', 'o', 'd', 'e'];
        let match_ins = Match::new(&query, &choice).unwrap();

        b.iter(|| match_ins.draw(false, false))
    }
}
