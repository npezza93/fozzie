use std::io::{stdout, Write};
use termion::raw::IntoRawMode;

pub fn print(text: String) {
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", text).unwrap();
    stdout.flush().unwrap();
}
