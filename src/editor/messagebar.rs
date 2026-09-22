use std::io::Result;

use super::{
    terminal::{Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct MessageBar {
    current_message: String,
    needs_redraw: bool,
    size: Size,
}

impl MessageBar {
    pub fn update_message(&mut self, new_message: String) {
        if self.current_message != new_message {
            self.current_message = new_message;
            self.mark_redraw(true);
        }
    }
}

impl UIComponent for MessageBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_y: usize) -> Result<()> {
        Terminal::print_row(origin_y, &self.current_message)
    }
}
