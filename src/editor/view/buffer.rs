use std::{fs, io::Result, path::Path};

use super::Location;
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

    pub fn insert(&mut self, at: Location, ch: char) {
        if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert(at.grapheme_index, ch);
        } else if self.height() == at.line_index {
            self.lines.push(Line::from(ch.to_string()));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }
}
