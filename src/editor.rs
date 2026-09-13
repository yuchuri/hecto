use std::{cmp::min, env, io::Result};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

mod terminal;
mod view;

use terminal::{Position, Size, Terminal};
use view::View;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    location: Position,
    view: View,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        self.handle_args();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn handle_args(&mut self) {
        if let Some(filename) = env::args_os().nth(1) {
            self.view.load(filename);
        }
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
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match code {
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
                | KeyCode::End => self.move_point(code)?,
                _ => {}
            },
            Event::Resize(width, height) => self.view.resize(Size {
                width: *width as usize,
                height: *height as usize,
            }),
            _ => {}
        }
        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<()> {
        Terminal::hide_caret()?;
        Terminal::move_caret_to(Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            Terminal::move_caret_to(self.location)?;
        }
        Terminal::show_caret()?;
        Terminal::execute()
    }
}
