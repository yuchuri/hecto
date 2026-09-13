use std::io::{self, Result, Write};

use crossterm::{
    cursor, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<()> {
        terminal::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()
    }

    pub fn terminate() -> Result<()> {
        Self::execute()?;
        terminal::disable_raw_mode()
    }

    pub fn clear_screen() -> Result<()> {
        queue!(io::stdout(), Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<()> {
        queue!(io::stdout(), Clear(ClearType::CurrentLine))
    }

    pub fn move_cursor_to(position: Position) -> Result<()> {
        queue!(io::stdout(), cursor::MoveTo(position.x, position.y))
    }

    pub fn hide_cursor() -> Result<()> {
        queue!(io::stdout(), cursor::Hide)
    }

    pub fn show_cursor() -> Result<()> {
        queue!(io::stdout(), cursor::Show)
    }

    pub fn print(msg: &str) -> Result<()> {
        queue!(io::stdout(), Print(msg))
    }

    pub fn size() -> Result<Size> {
        let (width, height) = terminal::size()?;
        Ok(Size { width, height })
    }

    pub fn execute() -> Result<()> {
        io::stdout().flush()
    }
}
