use std::{
    fs::{self, File},
    io::{Result, Write},
    path::Path,
};

use super::fileinfo::FileInfo;

use super::Location;
use crate::editor::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub dirty: bool,
}

impl Buffer {
    pub fn load(filename: impl AsRef<Path>) -> Result<Self> {
        let path = filename.as_ref();
        Ok(Self {
            lines: fs::read_to_string(path)?.lines().map(Line::from).collect(),
            file_info: FileInfo::from(filename),
            dirty: false,
        })
    }

    pub fn insert(&mut self, at: Location, ch: char) {
        if let Some(line) = self.lines.get_mut(at.line_idx) {
            line.insert(at.grapheme_idx, ch);
            self.dirty = true;
        } else if self.len() == at.line_idx {
            self.lines.push(Line::from(ch.to_string()));
            self.dirty = true;
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_idx == self.len() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            let newline = line.split_off(at.grapheme_idx);
            self.lines.insert(at.line_idx.saturating_add(1), newline);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Location) {
        if at.line_idx >= self.lines.len() {
            return;
        }

        if at.grapheme_idx < self.lines[at.line_idx].len() {
            self.lines[at.line_idx].remove(at.grapheme_idx);
            self.dirty = true;
        } else if at.grapheme_idx == self.lines[at.line_idx].len()
            && at.line_idx.saturating_add(1) < self.len()
        {
            let next_line = self.lines.remove(at.line_idx.saturating_add(1));
            self.lines[at.line_idx].append(next_line);
            self.dirty = true;
        }
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<()> {
        let file_info = FileInfo::from(file_name);
        self.save_to_file(&file_info)?;
        self.file_info = file_info;
        self.dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        self.save_to_file(&self.file_info)?;
        self.dirty = false;
        Ok(())
    }

    pub fn search(&self, query: &str, from: Location) -> Option<Location> {
        for (line_idx, line) in self.lines.iter().enumerate().skip(from.line_idx) {
            let from_grapheme_idx = if line_idx == from.line_idx {
                from.grapheme_idx
            } else {
                0
            };
            if let Some(grapheme_idx) = line.search(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }

        for (line_idx, line) in self.lines.iter().enumerate().take(from.line_idx) {
            if let Some(grapheme_idx) = line.search(query, 0) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}

impl Buffer {
    fn save_to_file(&self, file_info: &FileInfo) -> Result<()> {
        if let Some(file_path) = &file_info.get_path() {
            let mut file = File::create(file_path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        }
        Ok(())
    }
}
