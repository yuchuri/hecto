use std::{
    fmt::{Display, Write},
    mem,
    ops::{Deref, Index, Range},
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::Col;

pub type GraphemeIdx = usize;
pub type ByteIdx = usize;

#[derive(Default)]
pub struct Line {
    string: String,
    offsets: Vec<ByteIdx>,
    widths: Vec<Col>,
}
impl Line {
    pub fn get_visible_graphemes(&self, range: Range<Col>) -> LineView<'_> {
        if range.start >= range.end {
            return LineView {
                line: self,
                start_idx: 0,
                end_idx: 0,
                pad_left: false,
                pad_right: false,
            };
        }

        let mut start_idx = self.offsets.len();
        let mut end_idx = 0;
        let mut pad_left = false;
        let mut pad_right = false;

        for (idx, _) in self.offsets.iter().enumerate() {
            let current_col = if idx == 0 { 0 } else { self.widths[idx - 1] };
            let next_col = self.widths[idx];
            if current_col < range.start && next_col > range.start {
                pad_left = true;
            }
            if next_col > range.end {
                if current_col < range.end {
                    pad_right = true;
                }
                break;
            }
            if current_col >= range.start {
                start_idx = start_idx.min(idx);
                end_idx = idx + 1;
            }
        }

        LineView {
            line: self,
            start_idx,
            end_idx,
            pad_left,
            pad_right,
        }
    }

    // Inserts a character into the line, or appends it at the end if at == self.len() + 1
    pub fn insert(&mut self, at: GraphemeIdx, ch: char) {
        debug_assert!(at.saturating_sub(1) <= self.len());
        if let Some(&byte_idx) = self.offsets.get(at) {
            self.string.insert(byte_idx, ch);
        } else {
            self.string.push(ch);
        }
        *self = Line::from(mem::take(&mut self.string));
    }

    pub fn remove(&mut self, at: GraphemeIdx) {
        debug_assert!(at <= self.len());
        if let Some(&start_idx) = self.offsets.get(at) {
            let end_idx = self
                .offsets
                .get(at.saturating_add(1))
                .copied()
                .unwrap_or(self.string.len());
            self.string.drain(start_idx..end_idx);
            *self = Line::from(mem::take(&mut self.string));
        }
    }

    pub fn remove_last(&mut self) {
        self.remove(self.offsets.len().saturating_sub(1));
    }

    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.string);
        *self = Line::from(mem::take(&mut self.string));
    }

    pub fn append_char(&mut self, ch: char) {
        self.insert(self.len(), ch);
    }

    pub fn split_off(&mut self, at: GraphemeIdx) -> Self {
        debug_assert!(at <= self.len());
        if at >= self.len() {
            return Line::default();
        }

        let split_byte = self.grapheme_idx_to_byte_idx(at);
        let split_width = self.width_until(at);
        let right_string = self.string.split_off(split_byte);
        let mut right_offsets = self.offsets.split_off(at);
        let mut right_widths = self.widths.split_off(at);

        for (offset, width) in right_offsets.iter_mut().zip(right_widths.iter_mut()) {
            *offset -= split_byte;
            *width -= split_width;
        }

        Self {
            string: right_string,
            offsets: right_offsets,
            widths: right_widths,
        }
    }

    pub fn len(&self) -> GraphemeIdx {
        self.offsets.len()
    }

    pub fn width_until(&self, at: GraphemeIdx) -> Col {
        debug_assert!(at <= self.len());
        if at == 0 {
            0
        } else {
            self.widths.get(at - 1).copied().unwrap_or(self.width())
        }
    }

    pub fn width(&self) -> Col {
        self.widths.last().copied().unwrap_or_default()
    }

    pub fn search_forward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.len());
        if from_grapheme_idx >= self.len() {
            return None;
        }
        let start_byte_idx = self.grapheme_idx_to_byte_idx(from_grapheme_idx);
        self.string
            .get(start_byte_idx..)
            .and_then(|substr| substr.find(query))
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx.saturating_add(start_byte_idx)))
    }

    pub fn search_backward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.len());
        if from_grapheme_idx == 0 {
            return None;
        }
        let end_byte_idx = if from_grapheme_idx == self.len() {
            self.string.len()
        } else {
            self.grapheme_idx_to_byte_idx(from_grapheme_idx)
        };
        self.string
            .get(..end_byte_idx)
            .and_then(|substr| substr.match_indices(query).last())
            .map(|(byte_idx, _)| self.byte_idx_to_grapheme_idx(byte_idx))
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }
}

impl Line {
    fn replacement_character(grapheme: &str) -> Option<char> {
        let width = grapheme.width();
        match grapheme {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 && grapheme.trim().is_empty() => Some('␣'),
            _ if width == 0 => {
                if grapheme.chars().any(char::is_control) {
                    Some('▯')
                } else {
                    Some('·')
                }
            }
            _ => None,
        }
    }

    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        debug_assert!(grapheme_idx <= self.len());
        if grapheme_idx == 0 || self.is_empty() {
            return 0;
        }
        self.offsets.get(grapheme_idx).copied().unwrap_or_else(|| {
            #[cfg(debug_assertions)]
            {
                panic!("Fragment not found for grapheme index: {grapheme_idx:?}");
            }
            #[cfg(not(debug_assertions))]
            {
                0
            }
        })
    }

    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> GraphemeIdx {
        debug_assert!(byte_idx <= self.string.len());
        if byte_idx == 0 || self.is_empty() {
            return 0;
        }
        self.offsets
            .iter()
            .rposition(|&offset| offset <= byte_idx)
            .unwrap_or_else(|| {
                #[cfg(debug_assertions)]
                {
                    panic!("Fragment not found for byte index: {byte_idx:?}");
                }
                #[cfg(not(debug_assertions))]
                {
                    0
                }
            })
    }
}

impl From<String> for Line {
    fn from(line_str: String) -> Self {
        debug_assert!(line_str.is_empty() || line_str.lines().count() == 1);
        let mut offsets = Vec::with_capacity(line_str.len());
        let mut widths = Vec::with_capacity(line_str.len());

        let mut total_width = 0;
        for (byte_idx, grapheme) in line_str.grapheme_indices(true) {
            let width = match Self::replacement_character(grapheme) {
                Some(_) => 1,
                None => grapheme.width(),
            };
            total_width += width;
            offsets.push(byte_idx);
            widths.push(total_width);
        }
        Self {
            string: line_str,
            offsets,
            widths,
        }
    }
}

impl From<&str> for Line {
    fn from(line_str: &str) -> Self {
        Self::from(line_str.to_string())
    }
}

impl Index<GraphemeIdx> for Line {
    type Output = str;

    fn index(&self, idx: GraphemeIdx) -> &Self::Output {
        let start_byte = self.offsets[idx];
        let end_byte = self
            .offsets
            .get(idx + 1)
            .copied()
            .unwrap_or(self.string.len());
        &self.string[start_byte..end_byte]
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string)
    }
}

impl Deref for Line {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

pub struct LineView<'a> {
    line: &'a Line,
    start_idx: GraphemeIdx,
    end_idx: GraphemeIdx,
    pad_left: bool,
    pad_right: bool,
}

impl Display for LineView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pad_left {
            f.write_str("⋯")?;
        }
        for idx in self.start_idx..self.end_idx {
            let grapheme = &self.line[idx];
            if let Some(replacement) = Line::replacement_character(grapheme) {
                f.write_char(replacement)?;
            } else {
                f.write_str(grapheme)?;
            }
        }
        if self.pad_right {
            f.write_str("⋯")?;
        }
        Ok(())
    }
}
