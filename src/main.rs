#[macro_use]
extern crate clap;

use termion::event::Key;
use termion::input::TermRead;
use std::io::Write;

pub mod cursor;
pub mod settings;

fn main() {
    let settings = settings::parse();
    print_choices(&settings);

    let mut stdout = settings.stdout();

    write!(stdout, "{}", settings.prompt).unwrap();
    stdout.flush().unwrap();

    let tty = termion::get_tty().expect("err");
    for c in tty.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char(c) => write!(stdout, "{}", c).unwrap(),
            Key::Alt(c) => println!("^{}", c),
            Key::Ctrl(c) => println!("*{}", c),
            Key::Esc => println!("ESC"),
            Key::Left => println!("←"),
            Key::Right => println!("→"),
            Key::Up => println!("↑"),
            Key::Down => println!("↓"),
            Key::Backspace => println!("×"),
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
