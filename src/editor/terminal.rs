use std::io::{self, Result, Write};

use crossterm::{
    Command, cursor, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

/// Represents the Terminal.
/// Edge Case for platforms where `usize` < `u16`:
/// Regardless of the actual size of the Terminal, this representation
/// only spans over at most `usize::MAX` or `u16::size` rows/columns, whichever is smaller.
/// Each size returned truncates to min(`usize::MAX`, `u16::MAX`)
/// And should you attempt to set the caret out of these bounds, it will also be truncated.
pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<()> {
        terminal::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_caret_to(Position { x: 0, y: 0 })?;
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

    /// Moves the caret to the given Position.
    /// # Arguments
    /// * `Position` - the  `Position`to move the caret to. Will be truncated to `u16::MAX` if bigger.
    pub fn move_caret_to(position: Position) -> Result<()> {
        Self::queue_command(cursor::MoveTo(position.x as u16, position.y as u16))
    }

    pub fn hide_caret() -> Result<()> {
        Self::queue_command(cursor::Hide)
    }

    pub fn show_caret() -> Result<()> {
        Self::queue_command(cursor::Show)
    }

    pub fn print(string: &str) -> Result<()> {
        Self::queue_command(Print(string))
    }

    /// Returns the current size of this Terminal.
    /// Edge Case for systems with `usize` < `u16`:
    /// * A `Size` representing the terminal size. Any coordinate `z` truncated to `usize` if `usize` < `z` < `u16`
    pub fn size() -> Result<Size> {
        let (width, height) = terminal::size()?;
        Ok(Size {
            width: width as usize,
            height: height as usize,
        })
    }

    pub fn execute() -> Result<()> {
        io::stdout().flush()
    }

    fn queue_command(command: impl Command) -> Result<()> {
        queue!(io::stdout(), command)
    }
}
