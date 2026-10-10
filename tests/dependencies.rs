use fozzie::{choice::Choice, choices::Choices, config::Config, scorer::Score};
use regex::Regex;
use termion::{event::Key, input::TermRead};

fn config() -> Config {
    Config {
        lines: 10,
        prompt: "> ".into(),
        show_scores: false,
        query: None,
        delimiter: None,
        field: None,
        output: None,
        benchmark: false,
    }
}

#[test]
fn terminal_key_decoding_preserves_navigation_editing_and_unicode() {
    let cases: &[(&[u8], Key)] = &[
        (b"\x1b[A", Key::Up),
        (b"\x1b[B", Key::Down),
        (b"\x1b[C", Key::Right),
        (b"\x1b[D", Key::Left),
        (b"\x1bb", Key::Alt('b')),
        (b"\x1bf", Key::Alt('f')),
        (b"\x1bd", Key::Alt('d')),
        (b"\x1b\x7f", Key::Alt('\x7f')),
        (b"\x7f", Key::Backspace),
        (b"\x03", Key::Ctrl('c')),
        (b"\x04", Key::Ctrl('d')),
        (b"\x15", Key::Ctrl('u')),
        (b"\r", Key::Char('\n')),
        (b"\t", Key::Char('\t')),
        (b"\x1b", Key::Esc),
        ("é".as_bytes(), Key::Char('é')),
        ("💣".as_bytes(), Key::Char('💣')),
    ];
    for (bytes, expected) in cases {
        let keys = bytes.keys().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(keys, vec![*expected], "{bytes:?}");
    }
}

#[test]
fn regex_split_preserves_unicode_fields() {
    let mut config = config();
    config.delimiter = Some(Regex::new(r"\s+").unwrap());
    config.field = Some(0);
    config.output = Some(1);
    let choice = Choice::new("café\u{2003}東京", &config);
    assert_eq!(choice.searchable, "café");
    assert_eq!(choice.returnable, "東京");
    assert_eq!(choice.searchable_len, 4);
}

#[test]
fn parallel_ranking_preserves_input_order_for_equal_scores() {
    let mut config = config();
    config.delimiter = Some(Regex::new(" ").unwrap());
    config.field = Some(0);
    config.output = Some(1);
    let choices: Vec<_> = ["foo first", "foo second", "f_o_o third", "bar fourth"]
        .iter()
        .map(|line| Choice::new(line, &config))
        .collect();
    let mut matches = Choices::new(10, &choices, false);
    for _ in 0..10 {
        matches.filter(&['f', 'o', 'o']);
        for expected in ["first", "second", "third", "first"] {
            assert_eq!(matches.current_match().returnable, expected);
            matches.select_next();
        }
    }
}

#[test]
fn float_comparisons_preserve_negative_and_near_zero_highlight_positions() {
    let config = config();
    let choice = Choice::new("xxaxxbxx/", &config);
    let score = Score::new(&['a', 'b'], &choice);
    assert!(score.score < 0.0);
    assert_eq!(score.positions, [2, 5]);

    // The leading gap penalty crosses the filename bonus around zero.
    for prefix_len in [198, 199, 200] {
        let text = format!("{}/a", "x".repeat(prefix_len));
        let choice = Choice::new(&text, &config);
        let score = Score::new(&['a'], &choice);
        assert!(score.score.abs() < 0.006);
        assert_eq!(score.positions, [prefix_len + 1]);
    }
}
