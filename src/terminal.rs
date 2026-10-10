use raw_tty::IntoRawMode;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use termion::input::TermRead;

pub struct Terminal {
    pub output: File,
}

impl Terminal {
    pub fn new() -> Result<Terminal, Box<dyn Error>> {
        let output = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;

        Ok(Terminal { output })
    }

    pub fn keys(
        &mut self,
    ) -> Result<termion::input::Keys<raw_tty::RawReader<File>>, Box<dyn Error>> {
        Ok(self.output.try_clone()?.into_raw_mode()?.keys())
    }

    pub fn print(&mut self, text: &str) {
        self.output.write_all(text.as_bytes()).unwrap();
        self.output.flush().unwrap();
    }
}

/// Measure the terminal used for the UI, not the selected-result stdout pipe.
/// Read again on redraw so changes to the window size are picked up.
pub(crate) fn columns() -> Option<usize> {
    let tty = File::open("/dev/tty").ok()?;
    terminal_size::terminal_size_of(&tty)
        .map(|(terminal_size::Width(width), _)| usize::from(width).max(1))
}

pub(crate) fn cell_width(text: &str) -> usize {
    text.chars().map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum()
}

/// Return a UTF-8-safe prefix that fits in the available terminal cells.
pub(crate) fn clip(text: &str, columns: usize) -> &str {
    let mut used = 0;
    for (offset, character) in text.char_indices() {
        used += unicode_width::UnicodeWidthChar::width(character).unwrap_or(0);
        if used > columns {
            return &text[..offset];
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping_observes_display_cells_and_utf8_boundaries() {
        for (text, columns, expected) in [
            ("東京", 1, ""), ("東京", 2, "東"), ("東京", 3, "東"),
            ("東京", 4, "東京"), ("a💣b", 2, "a"), ("a💣b", 3, "a💣"),
            ("e\u{301}lan", 1, "e\u{301}"), ("café", 4, "café"),
        ] {
            assert_eq!(clip(text, columns), expected);
            assert!(cell_width(clip(text, columns)) <= columns);
        }
    }
}
