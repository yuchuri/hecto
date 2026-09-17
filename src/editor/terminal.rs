use std::{
    fmt::Display,
    io::{self, Result, Write},
};

use crossterm::{
    Command, cursor, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            col: self.col.saturating_sub(rhs.col),
            row: self.row.saturating_sub(rhs.row),
        }
    }
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
        Self::enter_alternate_screen()?;
        Self::clear_screen()?;
        Self::execute()
    }

    pub fn terminate() -> Result<()> {
        Self::leave_alternate_screen()?;
        Self::show_caret()?;
        Self::execute()?;
        terminal::disable_raw_mode()
    }

    pub fn enter_alternate_screen() -> Result<()> {
        Self::queue_command(terminal::EnterAlternateScreen)
    }

    pub fn leave_alternate_screen() -> Result<()> {
        Self::queue_command(terminal::LeaveAlternateScreen)
    }

    pub fn clear_screen() -> Result<()> {
        Self::queue_command(Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<()> {
        Self::queue_command(Clear(ClearType::CurrentLine))
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

    /// Moves the caret to the given Position.
    /// # Arguments
    /// * `position` - the `Position` to move the caret to. Will be truncated to `u16::MAX` if bigger.
    pub fn move_caret_to(position: Position) -> Result<()> {
        Self::queue_command(cursor::MoveTo(position.col as u16, position.row as u16))
    }

    pub fn hide_caret() -> Result<()> {
        Self::queue_command(cursor::Hide)
    }

    pub fn show_caret() -> Result<()> {
        Self::queue_command(cursor::Show)
    }

    pub fn print_row(row: usize, line: impl Display) -> Result<()> {
        Terminal::move_caret_to(Position { col: 0, row })?;
        Terminal::clear_line()?;
        Terminal::print(line)
    }

    pub fn print(text: impl Display) -> Result<()> {
        Self::queue_command(Print(text))
    }

    pub fn execute() -> Result<()> {
        io::stdout().flush()
    }

    fn queue_command(command: impl Command) -> Result<()> {
        queue!(io::stdout(), command)
    }
}
