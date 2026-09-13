use std::{fs, io::Result, path::Path};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(filename: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            lines: fs::read_to_string(filename)?
                .lines()
                .map(String::from)
                .collect(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
