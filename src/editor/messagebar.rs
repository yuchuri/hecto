use super::{terminal::Size, uicomponent::UIComponent};
use crate::editor::terminal::Terminal;
use std::io::Result;
use std::time::{Duration, Instant};

const DEFAULT_DURATION: Duration = Duration::from_secs(5);

struct Message {
    text: String,
    time: Instant,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            text: String::new(),
            time: Instant::now(),
        }
    }
}

impl Message {
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.time) > DEFAULT_DURATION
    }
}

#[derive(Default)]
pub struct MessageBar {
    current_message: Message,
    needs_redraw: bool,
    size: Size,
    cleared_after_expiry: bool, // ensures we can properly hide expired messages
}

impl MessageBar {
    pub fn update_message(&mut self, new_message: String) {
        self.current_message = Message {
            text: new_message,
            time: Instant::now(),
        };
        self.cleared_after_expiry = false;
        self.needs_redraw = true;
    }
}

impl UIComponent for MessageBar {
    fn set_need_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        (!self.cleared_after_expiry && self.current_message.is_expired()) || self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_y: usize) -> Result<()> {
        if self.current_message.is_expired() {
            self.cleared_after_expiry = true; // Upon expiration, we need to write out "" once to clear the message. To avoid clearing more than necessary, we  keep track of the fact that we've already cleared the expired message once.
        }
        let message = if self.current_message.is_expired() {
            ""
        } else {
            &self.current_message.text
        };
        Terminal::print_row(origin_y, message)
    }
}
