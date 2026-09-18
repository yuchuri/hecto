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
        } else if self.len() == at.line_index {
            self.lines.push(Line::from(ch.to_string()));
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_index == self.len() {
            self.lines.push(Line::default());
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let newline = line.split_off(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), newline);
        }
    }

    pub fn delete(&mut self, at: Location) {
        if at.grapheme_index < self.lines[at.line_index].len() {
            self.lines[at.line_index].remove(at.grapheme_index);
        } else if at.grapheme_index == self.lines[at.line_index].len()
            && at.line_index.saturating_add(1) < self.len()
        {
            let next_line = self.lines.remove(at.line_index.saturating_add(1));
            self.lines[at.line_index].append(next_line);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
