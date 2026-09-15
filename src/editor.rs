use std::{cmp::min, env, io::Result};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

mod terminal;
mod view;

use terminal::{Position, Size, Terminal};
use view::View;

#[derive(Clone, Copy, Default)]
struct Location {
    x: usize,
    y: usize,
}

pub struct Editor {
    should_quit: bool,
    location: Location,
    view: View,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;
        let mut view = View::default();
        if let Some(filename) = env::args_os().nth(1) {
            view.load(filename);
        }
        Ok(Self {
            should_quit: false,
            location: Location::default(),
            view,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match event::read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }
        }
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        self.view.render();
        let _ = Terminal::move_caret_to(Position {
            col: self.location.x,
            row: self.location.y,
        });
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    fn evaluate_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match code {
                KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => self.should_quit = true,
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End => self.move_point(code),
                _ => {}
            },
            Event::Resize(width, height) => self.view.resize(Size {
                width: width as usize,
                height: height as usize,
            }),
            _ => {}
        }
    }

    fn move_point(&mut self, key_code: KeyCode) {
        let Location { x, y } = &mut self.location;
        let Size { width, height } = Terminal::size().unwrap_or_default();
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
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye.\r\n");
        }
    }
}
