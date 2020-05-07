#![feature(test)]

extern crate test;

use fozzie::choices::Choices;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_filtering(b: &mut test::Bencher) {
        let choices: Vec<String> = vec![
            "CODE_OF_CONDUCT.md".to_string(),
            "Cargo.lock".to_string(),
            "Cargo.toml".to_string(),
            "LICENSE".to_string(),
            "README.md".to_string(),
            "benches/choices.rs".to_string(),
            "benches/drawing.rs".to_string(),
            "benches/matching.rs".to_string(),
            "benches/scoring.rs".to_string(),
            "src/bonus.rs".to_string()
        ];
        let query = ['c', 'o', 'd', 'e'];

        b.iter(|| Choices::new(10, &choices, false).filter(&query))
    }
}
