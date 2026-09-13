use std::io::{self, Result, Write};

use crossterm::{
    Command, cursor, queue,
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
        Self::queue_command(Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<()> {
        Self::queue_command(Clear(ClearType::CurrentLine))
    }

    pub fn move_cursor_to(position: Position) -> Result<()> {
        Self::queue_command(cursor::MoveTo(position.x, position.y))
    }

    pub fn hide_cursor() -> Result<()> {
        Self::queue_command(cursor::Hide)
    }

    pub fn show_cursor() -> Result<()> {
        Self::queue_command(cursor::Show)
    }

    pub fn print(string: &str) -> Result<()> {
        Self::queue_command(Print(string))
    }

    pub fn size() -> Result<Size> {
        let (width, height) = terminal::size()?;
        Ok(Size { width, height })
    }

    pub fn execute() -> Result<()> {
        io::stdout().flush()
    }

    fn queue_command(command: impl Command) -> Result<()> {
        queue!(io::stdout(), command)
    }
}
