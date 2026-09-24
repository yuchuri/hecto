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

#[derive(Eq, PartialEq, Default)]
enum PromptType {
    Save,
    Search,
    #[default]
    None,
}

impl PromptType {
    fn is_none(&self) -> bool {
        *self == PromptType::None
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    prompt_type: PromptType,
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
        editor.handle_resize_command(size);
        editor
            .message_bar
            .update_message("HELP: Ctrl-F = search | Ctrl-S = save | Ctrl-Q = quit");

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

        if self.in_prompt() {
            self.command_bar.render(bottom_bar_row);
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

        let new_caret_pos = if self.in_prompt() {
            Position {
                row: bottom_bar_row,
                col: self.command_bar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };

        let _ = Terminal::move_caret_to(new_caret_pos);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.filename);
        self.status_bar.update_status(status);
        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
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
        if let Command::System(System::Resize(size)) = command {
            self.handle_resize_command(size);
            return;
        }

        match self.prompt_type {
            PromptType::Save => self.process_command_during_save(command),
            PromptType::Search => self.process_command_during_search(command),
            PromptType::None => self.process_command_no_prompt(command),
        }
    }

    fn process_command_no_prompt(&mut self, command: Command) {
        if matches!(command, Command::System(System::Quit)) {
            self.handle_quit_command();
            return;
        }
        self.reset_quit_time();

        match command {
            Command::System(System::Search) => self.set_prompt(PromptType::Search),
            Command::System(System::Save) => self.handle_save_command(),
            Command::Move(move_command) => self.view.handle_move_command(move_command),
            Command::Edit(edit_command) => self.view.handle_edit_command(edit_command),
            _ => {}
        }
    }

    fn handle_resize_command(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            width: size.width,
            height: size.height.saturating_sub(2),
        });
        let bar_size = Size {
            width: size.width,
            height: 1,
        };
        self.status_bar.resize(bar_size);
        self.message_bar.resize(bar_size);
        self.command_bar.resize(bar_size);
    }

    fn handle_save_command(&mut self) {
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }

    fn process_command_during_save(&mut self, command: Command) {
        match command {
            Command::System(System::Dismiss) => {
                self.set_prompt(PromptType::None);
                self.message_bar.update_message("Save aborted.");
            }
            Command::Edit(Edit::InsertNewLine) => {
                let filename = self.command_bar.value();
                self.save(Some(&filename));
                self.set_prompt(PromptType::None);
            }
            Command::Edit(edit_command) => self.command_bar.handle_edit_command(edit_command),
            _ => {}
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

    fn process_command_during_search(&mut self, command: Command) {
        match command {
            Command::System(System::Dismiss) | Command::Edit(Edit::InsertNewLine) => {
                self.set_prompt(PromptType::None);
            }
            Command::Edit(edit_command) => self.command_bar.handle_edit_command(edit_command),
            _ => {}
        }
    }

    fn handle_quit_command(&mut self) {
        let is_modified = self.view.get_status().is_modified;
        if !is_modified || self.quit_time + 1 == QUIT_TIMES {
            self.should_quit = true;
        } else if is_modified {
            self.quit_time += 1;
            self.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} times to quit.",
                QUIT_TIMES - self.quit_time
            ));
        }
    }

    fn reset_quit_time(&mut self) {
        if self.quit_time > 0 {
            self.quit_time = 0;
            self.update_message("");
        }
    }

    fn update_message(&mut self, message: &str) {
        self.message_bar.update_message(message);
    }

    fn in_prompt(&self) -> bool {
        !self.prompt_type.is_none()
    }

    fn set_prompt(&mut self, prompt: PromptType) {
        match prompt {
            PromptType::None => self.message_bar.set_needs_redraw(true),
            PromptType::Save => self.command_bar.set_prompt("Save as: "),
            PromptType::Search => self.command_bar.set_prompt("Search: "),
        }
        self.command_bar.clear_value();
        self.prompt_type = prompt;
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
