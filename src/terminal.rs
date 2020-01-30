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
