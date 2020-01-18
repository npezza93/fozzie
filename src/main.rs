#[macro_use]
extern crate clap;

use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;

pub mod cursor;
pub mod search;
pub mod settings;

fn main() {
    let settings = settings::parse();
    let mut search = search::Search::new();

    print_choices(&settings);

    let mut stdout = settings.stdout();

    search.render(&settings);

    let tty = termion::get_tty().expect("err");
    for c in tty.keys() {
        match c.unwrap() {
            Key::Char(c) => search.keypress(c, &settings),
            Key::Alt(c) => println!("^{}", c),
            Key::Ctrl(c) => println!("*{}", c),
            Key::Esc => break,
            Key::Left => search.left(&settings),
            Key::Right => search.right(&settings),
            Key::Up => println!("↑"),
            Key::Down => println!("↓"),
            Key::Backspace => search.backspace(&settings),
            _ => {}
        }
        stdout.flush().unwrap();
    }
}

fn print_choices(settings: &settings::Settings) {
    let mut stdout = settings.stdout();

    cursor::move_screen_up(settings.lines);
    write!(stdout, "\n\r").unwrap();
    settings.choices[0..settings.lines].iter().for_each(|line| {
        write!(stdout, "{}\n\r", line).unwrap();
    });

    let printed_options_count = [settings.choices.len(), settings.lines];
    if let Some(lines) = printed_options_count.iter().min() {
        cursor::up(lines + 1);
    };
}
