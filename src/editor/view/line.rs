use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

pub struct Line {
    string: String,
}

impl Line {
    pub fn get(&self, range: Range<usize>) -> &str {
        if range.start >= range.end {
            return "";
        }
        let mut start = self.string.len();
        let mut end = start;
        for (index, (byte_index, _)) in self.string.grapheme_indices(true).enumerate() {
            if index == range.start {
                start = byte_index;
            }
            if index == range.end {
                end = byte_index;
                break;
            }
        }
        self.string.get(start..end).unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.string.graphemes(true).count()
    }
}

impl From<&str> for Line {
    fn from(line_str: &str) -> Self {
        Line {
            string: String::from(line_str),
        }
    }
}
