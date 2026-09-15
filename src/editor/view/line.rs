use std::{cmp, ops::Range};

pub struct Line {
    string: String,
}

impl Line {
    pub fn get(&self, range: Range<usize>) -> &str {
        let start = range.start;
        let end = cmp::min(range.end, self.string.len());
        self.string.get(start..end).unwrap_or_default()
    }
}

impl From<&str> for Line {
    fn from(line_str: &str) -> Self {
        Line {
            string: String::from(line_str),
        }
    }
}
