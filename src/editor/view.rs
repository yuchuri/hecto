use std::path::Path;

use super::terminal::{Size, Terminal};

mod buffer;

use buffer::Buffer;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
        }
    }
}

impl View {
    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.needs_redraw = true;
    }

    fn render_line(at: usize, line: &str) {
        let result = Terminal::print_row(at, line);
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }
        let Size { width, height } = self.size;
        if width == 0 || height == 0 {
            return;
        }
        // We allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // It's allowed to be a bit up or down
        let vertical_center = height / 3;

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row) {
                let truncated_line = if line.len() > width {
                    &line[..width]
                } else {
                    line
                };
                Self::render_line(current_row, truncated_line);
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width));
            } else {
                Self::render_line(current_row, "~");
            }
        }
        self.needs_redraw = false;
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return " ".into();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        if width <= len {
            return "~".into();
        }

        // We allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // It's allowed to be a bit to the left or right.
        let padding = width.saturating_sub(len) / 2;
        let space = " ".repeat(padding.saturating_sub(1));
        let mut welcome_message = format!("~{space}{welcome_message}");
        welcome_message.truncate(width);
        welcome_message
    }

    pub fn load(&mut self, filename: impl AsRef<Path>) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }
}
