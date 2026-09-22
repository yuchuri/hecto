use std::{env, io::Result};

use crossterm::event::{self, Event};

mod documentstatus;
mod editorcommand;
mod fileinfo;
mod messagebar;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use editorcommand::EditorCommand;
use messagebar::MessageBar;
use statusbar::StatusBar;
use terminal::{Size, Terminal};
use uicomponent::UIComponent;
use view::View;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    terminal_size: Size,
    title: String,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;
        let mut editor = Editor::default();
        let size = Terminal::size().unwrap_or_default();
        editor.resize(size);

        if let Some(filename) = env::args_os().nth(1) {
            editor.view.load(filename);
        }
        editor
            .message_bar
            .update_message("HELP: Ctrl-S = save | Ctrl-Q = quit".into());
        editor.refresh_status();
        Ok(editor)
    }

    fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            width: size.width,
            height: size.height.saturating_sub(2),
        });
        self.status_bar.resize(Size {
            width: size.width,
            height: 1,
        });
        self.message_bar.resize(Size {
            width: size.width,
            height: 1,
        });
    }

    fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.filename);
        self.status_bar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
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
            let status = self.view.get_status();
            self.status_bar.update_status(status);
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.width == 0 || self.terminal_size.height == 0 {
            return;
        }
        let _ = Terminal::hide_caret();
        self.message_bar
            .render(self.terminal_size.height.saturating_sub(1));
        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }
        if self.terminal_size.height > 2 {
            self.view.render(0);
        }
        let _ = Terminal::move_caret_to(self.view.caret_position());
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    fn evaluate_event(&mut self, event: Event) {
        if let Ok(command) = EditorCommand::try_from(event) {
            if matches!(command, EditorCommand::Quit) {
                self.should_quit = true;
            } else if let EditorCommand::Resize(size) = command {
                self.resize(size)
            } else {
                self.view.handle_command(command);
            }
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
