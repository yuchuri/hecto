use std::io::{self, Result};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};
pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<()> {
        terminal::enable_raw_mode()?;
        Self::clear_screen()
    }

    pub fn terminate() -> Result<()> {
        terminal::disable_raw_mode()
    }

    pub fn clear_screen() -> Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))
    }

    pub fn move_cursor_to(x: u16, y: u16) -> Result<()> {
        execute!(io::stdout(), MoveTo(x, y))
    }

    pub fn size() -> Result<(u16, u16)> {
        terminal::size()
    }
}
