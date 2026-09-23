use super::{command::Edit, line::Line, size::Size, terminal::Terminal, uicomponent::UIComponent};
use std::io::Result;

#[derive(Default)]
pub struct CommandBar {
    prompt: String,
    value: Line,
    needs_redraw: bool,
    size: Size,
}

impl CommandBar {
    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(ch) => self.value.append_char(ch),
            Edit::InsertNewLine | Edit::Delete => {}
            Edit::DeleteBackward => self.value.remove_last(),
        }
        self.set_needs_redraw(true);
    }

    pub fn caret_position_col(&self) -> usize {
        self.prompt
            .len()
            .saturating_add(self.value.len())
            .min(self.size.width)
    }

    pub fn value(&self) -> String {
        self.value.to_string()
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
    }
}

impl UIComponent for CommandBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_row: usize) -> Result<()> {
        // This is how much space there is between the right side of the prompt and the edge of the
        // bar.
        let area_for_value = self.size.width.saturating_sub(self.prompt.len());
        // We always want to show the left part of the value, therefore the end of the visible range
        // we try to access will be equal to the full width.
        let value_end = self.value.len();
        // This should give us the star for the grapheme subrange we want to print out.
        let value_start = value_end.saturating_sub(area_for_value);
        let message = format!(
            "{}{}",
            self.prompt,
            self.value.get_visible_graphemes(value_start..value_end)
        );
        let to_print = if message.len() <= self.size.width {
            message
        } else {
            String::new()
        };
        Terminal::print_row(origin_row, &to_print)
    }
}
