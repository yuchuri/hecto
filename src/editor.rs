use std::{cmp::min, io::Result};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

mod terminal;

use terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    location: Position,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<()> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = event::read()?;
            self.evaluate_event(&event)?;
        }
        Ok(())
    }

    fn move_point(&mut self, key_code: &KeyCode) -> Result<()> {
        let Position { x, y } = &mut self.location;
        let Size { width, height } = Terminal::size()?;
        match key_code {
            KeyCode::Up => *y = y.saturating_sub(1),
            KeyCode::Down => *y = min(y.saturating_add(1), height.saturating_sub(1)),
            KeyCode::Left => *x = x.saturating_sub(1),
            KeyCode::Right => *x = min(x.saturating_add(1), width.saturating_sub(1)),
            KeyCode::PageUp => *y = 0,
            KeyCode::PageDown => *y = height.saturating_sub(1),
            KeyCode::Home => *x = 0,
            KeyCode::End => *x = width.saturating_sub(1),
            _ => {}
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind,
            ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true
                }
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End
                    if *kind == KeyEventKind::Press =>
                {
                    self.move_point(code)?
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<()> {
        Terminal::hide_caret()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            Self::draw_rows()?;
            Terminal::move_caret_to(self.location)?;
        }
        Terminal::show_caret()?;
        Terminal::execute()
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

    fn draw_rows() -> Result<()> {
        let Size { height, .. } = Terminal::size()?;
        for current_row in 0..height {
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
}
