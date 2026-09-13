use std::io::{self, Result};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn run(&mut self) {
        Self::initialize().unwrap();
        let result = self.repl();
        Self::terminate().unwrap();
        result.unwrap();
    }

    fn initialize() -> Result<()> {
        terminal::enable_raw_mode()?;
        Self::clean_screen()
    }

    fn terminate() -> Result<()> {
        terminal::disable_raw_mode()
    }

    fn clean_screen() -> Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))
    }

    fn repl(&mut self) -> Result<()> {
        loop {
            let event = event::read()?;
            self.evaluate_event(&event)?;
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
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
        if self.should_quit {
            Self::clean_screen()?;
            print!("Goodbye.\r\n");
        }
        Ok(())
    }
}
