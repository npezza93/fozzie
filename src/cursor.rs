use crate::settings;
use std::io::Write;
use termion::clear;

pub fn up(n: usize) {
    print!("\x1B[{}A", n);
}

pub fn left(n: usize, settings: &settings::Settings) {
    write!(settings.stdout(), "\x1B[{}D", n).unwrap();
}

pub fn right(n: usize, settings: &settings::Settings) {
    write!(settings.stdout(), "\x1B[{}C", n).unwrap();
}

pub fn clear_char(n: usize, settings: &settings::Settings) {
    write!(settings.stdout(), "\x1B[{}X", n).unwrap();
}

pub fn clear_line(settings: &settings::Settings) {
    write!(settings.stdout(), "{}", clear::CurrentLine).unwrap();
}

pub fn col(n: usize, settings: &settings::Settings) {
    write!(settings.stdout(), "\x1B[{}G", n).unwrap();
}

pub fn move_screen_up(n: usize) {
    print!("{}", "\n".repeat(n));
    up(n);
}
