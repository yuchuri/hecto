#[derive(Debug, Default, Eq, PartialEq)]
pub struct DocumentStatus {
    pub total_lines: usize,
    pub current_line_index: usize,
    pub is_modified: bool,
    pub filename: String,
}

impl DocumentStatus {
    pub fn modified_indicator_to_string(&self) -> String {
        if self.is_modified {
            String::from("(modified)")
        } else {
            String::new()
        }
    }

    pub fn line_count_to_string(&self) -> String {
        format!("{} line", self.total_lines)
    }

    pub fn position_indicator_to_string(&self) -> String {
        format!("{}/{}", self.current_line_index, self.total_lines)
    }
}
