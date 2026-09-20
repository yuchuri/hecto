use std::{
    fs::{self, File},
    io::{Result, Write},
    path::{Path, PathBuf},
};

use super::Location;
use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

impl Buffer {
    pub fn load(filename: impl AsRef<Path>) -> Result<Self> {
        let path = filename.as_ref();
        Ok(Self {
            lines: fs::read_to_string(path)?.lines().map(Line::from).collect(),
            path: Some(path.to_path_buf()),
            dirty: false,
        })
    }

    pub fn insert(&mut self, at: Location, ch: char) {
        if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert(at.grapheme_index, ch);
            self.dirty = true;
        } else if self.len() == at.line_index {
            self.lines.push(Line::from(ch.to_string()));
            self.dirty = true;
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_index == self.len() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let newline = line.split_off(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), newline);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Location) {
        if at.grapheme_index < self.lines[at.line_index].len() {
            self.lines[at.line_index].remove(at.grapheme_index);
            self.dirty = true;
        } else if at.grapheme_index == self.lines[at.line_index].len()
            && at.line_index.saturating_add(1) < self.len()
        {
            let next_line = self.lines.remove(at.line_index.saturating_add(1));
            self.lines[at.line_index].append(next_line);
            self.dirty = true;
        }
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(filename) = &self.path {
            let mut file = File::create(filename)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
