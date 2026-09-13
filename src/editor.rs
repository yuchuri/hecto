use std::io::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

mod terminal;

use terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
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

    fn evaluate_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<()> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            print!("Goodbye.\r\n");
        } else {
            Self::draw_rows()?;
            Terminal::move_cursor_to(Position { x: 0, y: 0 })?;
        }
        Terminal::show_cursor()?;
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
