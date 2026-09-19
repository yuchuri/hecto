use std::{
    env,
    fmt::{Display, Write},
    mem,
    ops::{Index, Range},
    sync::LazyLock,
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct Line {
    string: String,
    offsets: Vec<usize>,
    widths: Vec<usize>,
}

impl Line {
    pub fn get_visible_graphemes(&self, range: Range<usize>) -> LineView<'_> {
        if range.start >= range.end {
            return LineView {
                line: self,
                start_index: 0,
                end_index: 0,
                pad_left: false,
                pad_right: false,
            };
        }

        let mut start_index = self.offsets.len();
        let mut end_index = 0;
        let mut pad_left = false;
        let mut pad_right = false;

        for (index, _) in self.offsets.iter().enumerate() {
            let current_col = if index == 0 {
                0
            } else {
                self.widths[index - 1]
            };
            let next_col = self.widths[index];
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
                start_index = start_index.min(index);
                end_index = index + 1;
            }
        }

        LineView {
            line: self,
            start_index,
            end_index,
            pad_left,
            pad_right,
        }
    }

    pub fn insert(&mut self, at: usize, ch: char) {
        let byte_index = self.offsets.get(at).copied().unwrap_or(self.string.len());
        self.string.insert(byte_index, ch);
        *self = Line::from(mem::take(&mut self.string));
    }

    pub fn remove(&mut self, at: usize) {
        if let Some(&start_index) = self.offsets.get(at) {
            let end_index = self
                .offsets
                .get(at.saturating_add(1))
                .copied()
                .unwrap_or(self.string.len());
            self.string.drain(start_index..end_index);
            *self = Line::from(mem::take(&mut self.string));
        }
    }

    pub fn append(&mut self, other: Line) {
        self.string.push_str(&other.string);
        *self = Line::from(mem::take(&mut self.string));
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        if at > self.len() {
            return Line::default();
        }

        let split_byte = self.offsets.get(at).copied().unwrap_or(self.string.len());
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

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    pub fn width_until(&self, at: usize) -> usize {
        if at == 0 {
            0
        } else {
            self.widths.get(at - 1).copied().unwrap_or(self.width())
        }
    }

    pub fn width(&self) -> usize {
        self.widths.last().copied().unwrap_or_default()
    }

    fn replacement_character(grapheme: &str) -> Option<char> {
        let width = grapheme_width(grapheme);
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
}

impl From<String> for Line {
    fn from(line_str: String) -> Self {
        let mut offsets = Vec::with_capacity(line_str.len());
        let mut widths = Vec::with_capacity(line_str.len());

        let mut total_width = 0;
        for (byte_index, grapheme) in line_str.grapheme_indices(true) {
            let width = match Self::replacement_character(grapheme) {
                Some(_) => 1,
                None => grapheme_width(grapheme),
            };
            total_width += width;
            offsets.push(byte_index);
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

impl Index<usize> for Line {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        let start_byte = self.offsets[index];
        let end_byte = self
            .offsets
            .get(index + 1)
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

pub struct LineView<'a> {
    line: &'a Line,
    start_index: usize,
    end_index: usize,
    pad_left: bool,
    pad_right: bool,
}

impl Display for LineView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pad_left {
            f.write_str("⋯")?;
        }
        for index in self.start_index..self.end_index {
            let grapheme = &self.line[index];
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

static USE_CJK_WIDTH: LazyLock<bool> = LazyLock::new(|| {
    if let Ok(val) = env::var("HECTO_CJK") {
        return val == "1" || val.eq_ignore_ascii_case("true");
    }
    for var in ["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(locale) = env::var(var) {
            let locale = locale.to_ascii_lowercase();
            if locale.starts_with("zh") || locale.contains("ja") || locale.contains("ko") {
                return true;
            }
        }
    }
    false
});

fn grapheme_width(grapheme: &str) -> usize {
    if *USE_CJK_WIDTH {
        grapheme.width_cjk()
    } else {
        grapheme.width()
    }
}
