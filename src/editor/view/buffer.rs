use std::{fs, io::Result, path::Path};

use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn load(filename: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            lines: fs::read_to_string(filename)?
                .lines()
                .map(Line::from)
                .collect(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }
}
