use std::io::Result;

use super::terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View;

impl View {
    pub fn render() -> Result<()> {
        Terminal::move_caret_to(Position::default())?;
        let Size { height, .. } = Terminal::size()?;
        Terminal::clear_line()?;
        Terminal::print("Hello, World!\r\n")?;
        for current_row in 1..height {
            Terminal::clear_line()?;

            // We allow this since we don't care if our welcome message is put _exactly_ in the middle.
            // It's allowed to be a bit up or down
            if current_row == height / 3 {
                Self::draw_welcome_message()?;
            } else {
                Self::draw_empty_row()?;
            }
            if current_row + 1 < height {
                Terminal::print("\r\n")?;
            }
        }
        Ok(())
    }

    fn draw_welcome_message() -> Result<()> {
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let width = Terminal::size()?.width;
        let len = welcome_message.len();

        // We allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // It's allowed to be a bit to the left or right.
        let padding = width.saturating_sub(len) / 2;
        let space = " ".repeat(padding.saturating_sub(1));
        let mut welcome_message = format!("~{space}{welcome_message}");
        welcome_message.truncate(width);
        Terminal::print(&welcome_message)
    }

    fn draw_empty_row() -> Result<()> {
        Terminal::print("~")
    }
}
