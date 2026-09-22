use std::{fmt::Display, io::Result, path::Path};

use unicode_width::UnicodeWidthStr;

use super::{
    NAME, VERSION,
    documentstatus::DocumentStatus,
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};

mod buffer;
mod line;

use crate::editor::uicomponent::UIComponent;
use buffer::Buffer;
use line::Line;

#[derive(Clone, Copy, Default)]
struct Location {
    grapheme_index: usize,
    line_index: usize,
}

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    // The view always starts at`{0/0}`. The `size` property determines the visible area
    size: Size,
    target_grapheme_index: usize,
    text_location: Location,
    scroll_offset: Position,
}

impl View {
    pub fn load(&mut self, filename: impl AsRef<Path>) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.mark_redraw(true);
        }
    }

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.len(),
            current_line_index: self.text_location.line_index,
            is_modified: self.buffer.dirty,
            filename: self.buffer.file_info.to_string(),
        }
    }

    pub fn save(&mut self) {
        let _ = self.buffer.save();
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(_) | EditorCommand::Quit => {}
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Insert(ch) => self.insert(ch),
            EditorCommand::Backspace => self.backspace(),
            EditorCommand::Delete => self.delete(),
            EditorCommand::Enter => self.insert_newline(),
            EditorCommand::Save => self.save(),
        }
    }

    fn move_text_location(&mut self, direction: Direction) {
        let height = self.size.height;
        // This match moves the positon, but does not check for all boundaries.
        // The final boundarline checking happens after the match statement.
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }
        self.scroll_text_location_into_view();
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.snap_to_valid_line();
        self.snap_to_valid_grapheme();
    }

    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index -= 1;
            self.target_grapheme_index = self.text_location.grapheme_index;
        } else if self.text_location.line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::len);
        if self.text_location.grapheme_index < line_width {
            self.text_location.grapheme_index += 1;
            self.target_grapheme_index = self.text_location.grapheme_index;
        } else {
            self.move_down(1);
            self.move_to_start_of_line();
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
        self.target_grapheme_index = self.text_location.grapheme_index;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::len);
        self.target_grapheme_index = self.text_location.grapheme_index;
    }

    // Ensures self.location.grapheme_index points to a valid grapheme index by snapping it to the left most grapheme if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| line.len().min(self.target_grapheme_index));
    }

    // Ensures self.location.line_index points to a valid line index by snapping it to the bottom most line if appropriate.
    // Doesn't trigger scrolling.
    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = self.buffer.len().min(self.text_location.line_index);
    }

    fn insert(&mut self, ch: char) {
        let old_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::len);
        self.buffer.insert(self.text_location, ch);
        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::len);
        if new_len > old_len {
            self.move_text_location(Direction::Right);
        }
        self.target_grapheme_index = self.text_location.grapheme_index;
        self.scroll_text_location_into_view();
        self.mark_redraw(true);
    }

    fn backspace(&mut self) {
        if self.text_location.line_index == 0 && self.text_location.grapheme_index == 0 {
            return;
        }
        self.move_text_location(Direction::Left);
        self.delete();
    }

    fn delete(&mut self) {
        self.buffer.delete(self.text_location);
        self.scroll_text_location_into_view();
        self.mark_redraw(true);
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.move_text_location(Direction::Right);
        self.mark_redraw(true);
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { col, row } = self.text_location_to_position();
        let Size { width, height } = self.size;
        let mut offset_changed = false;

        // Scroll vertically
        if row < self.scroll_offset.row {
            self.scroll_offset.row = row;
            offset_changed = true;
        } else if row >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = row.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        // Scroll horizontally
        if col < self.scroll_offset.col {
            self.scroll_offset.col = col;
            offset_changed = true;
        } else if col >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = col.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }
        if offset_changed {
            self.mark_redraw(true);
        }
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });
        Position { col, row }
    }

    fn render_line(at: usize, line: impl Display) -> Result<()> {
        Terminal::print_row(at, line)
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let message_width = welcome_message.width();
        let remaining_width = width.saturating_sub(1);

        // Hide the welcome message if it doesn't fit completely within the available width.
        if message_width + 1 > width {
            return "~".into();
        }

        format!("{:<1}{welcome_message:^remaining_width$}", "~")
    }
}

impl UIComponent for View {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }

    fn draw(&mut self, origin_y: usize) -> std::io::Result<()> {
        let Size { width, height } = self.size;
        let end_y = origin_y.saturating_add(height);

        // We allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // It's allowed to be a bit up or down
        let top_third = height / 3;
        let scroll_top = self.scroll_offset.row;
        let left = self.scroll_offset.col;
        let right = left.saturating_add(width);

        for current_row in origin_y..end_y {
            // to get the correct line index, we have to take current_row (the absolute row on screen),
            // subtract origin_y to get the current row relative to the view (ranging from 0 to self.size.height)
            // and add the scroll offset.
            let view_row = current_row.saturating_sub(origin_y);

            if let Some(line) = self.buffer.lines.get(view_row.saturating_add(scroll_top)) {
                Self::render_line(current_row, line.get_visible_graphemes(left..right))?;
            } else if current_row == origin_y.saturating_add(top_third) && self.buffer.is_empty() {
                Self::render_line(current_row, Self::build_welcome_message(width))?;
            } else {
                Self::render_line(current_row, "~")?;
            }
        }

        Ok(())
    }
}
