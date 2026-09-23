use std::{env, io::Result};

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

mod command;
mod commandbar;
mod documentstatus;
mod line;
mod messagebar;
mod position;
mod size;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use crate::editor::command::Edit;
use crate::editor::position::Position;
use command::{Command, System};
use commandbar::CommandBar;
use messagebar::MessageBar;
use size::Size;
use statusbar::StatusBar;
use terminal::Terminal;
use uicomponent::UIComponent;
use view::View;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 3;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: Option<CommandBar>,
    terminal_size: Size,
    title: String,
    quit_time: u8,
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
        editor
            .message_bar
            .update_message("HELP: Ctrl-S = save | Ctrl-Q = quit");

        if let Some(path) = env::args_os().nth(1)
            && editor.view.load(&path).is_err()
        {
            editor.message_bar.update_message(&format!(
                "ERR: Could not open file: {}",
                path.to_string_lossy()
            ))
        }
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
        if let Some(command_bar) = self.command_bar.as_mut() {
            command_bar.resize(Size {
                height: 1,
                width: size.width,
            });
        }
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
        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        let _ = Terminal::hide_caret();
        if let Some(command_bar) = self.command_bar.as_mut() {
            command_bar.render(bottom_bar_row);
        } else {
            self.message_bar.render(bottom_bar_row);
        }
        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }
        if self.terminal_size.height > 2 {
            self.view.render(0);
        }
        let new_caret_pos = if let Some(commandbar) = self.command_bar.as_mut() {
            Position {
                row: bottom_bar_row,
                col: commandbar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };
        let _ = Terminal::move_caret_to(new_caret_pos);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    fn evaluate_event(&mut self, event: Event) {
        let should_process = match event {
            Event::Key(KeyEvent { kind, .. }) => kind == KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process && let Ok(command) = Command::try_from(event) {
            self.process_command(command);
        }
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::System(System::Quit) => {
                if self.command_bar.is_none() {
                    self.handle_quit();
                }
            }
            Command::System(System::Resize(size)) => self.resize(size),
            _ => self.reset_quit_time(),
        }

        match command {
            Command::System(System::Quit | System::Resize(_)) => {}
            Command::System(System::Save) => {
                if self.command_bar.is_none() {
                    self.handle_save()
                }
            }
            Command::System(System::Dismiss) => {
                if self.command_bar.is_some() {
                    self.dismiss_prompt();
                    self.message_bar.update_message("Save aborted.");
                }
            }
            Command::Edit(edit_command) => {
                if let Some(command_bar) = self.command_bar.as_mut() {
                    if matches!(edit_command, Edit::InsertNewLine) {
                        let file_name = command_bar.value();
                        self.dismiss_prompt();
                        self.save(Some(&file_name))
                    } else {
                        command_bar.handle_edit_command(edit_command);
                    }
                } else {
                    self.view.handle_edit_command(edit_command);
                }
            }
            Command::Move(move_command) => {
                if self.command_bar.is_none() {
                    self.view.handle_move_command(move_command);
                }
            }
        }
    }

    fn dismiss_prompt(&mut self) {
        self.command_bar = None;
        self.message_bar.set_needs_redraw(true);
    }

    fn show_prompt(&mut self) {
        let mut command_bar = CommandBar::default();
        command_bar.set_prompt("Save as: ");
        command_bar.resize(Size {
            width: self.terminal_size.width,
            height: 1,
        });
        command_bar.set_needs_redraw(true);
        self.command_bar = Some(command_bar);
    }

    fn handle_save(&mut self) {
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.show_prompt();
        }
    }

    fn save(&mut self, file_name: Option<&str>) {
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save()
        };
        if result.is_ok() {
            self.message_bar.update_message("File saved successfully.");
        } else {
            self.message_bar.update_message("Error writing file!");
        }
    }

    fn handle_quit(&mut self) {
        let is_modified = self.view.get_status().is_modified;
        if !is_modified || self.quit_time + 1 == QUIT_TIMES {
            self.should_quit = true;
        } else if is_modified {
            self.quit_time += 1;
            self.message_bar.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} times to quit.",
                QUIT_TIMES - self.quit_time
            ));
        }
    }

    fn reset_quit_time(&mut self) {
        if self.quit_time > 0 {
            self.quit_time = 0;
            self.message_bar.update_message("");
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
