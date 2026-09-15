use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::terminal::Size;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match code {
                KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => Ok(EditorCommand::Quit),
                KeyCode::Up => Ok(EditorCommand::Move(Direction::Up)),
                KeyCode::Down => Ok(EditorCommand::Move(Direction::Down)),
                KeyCode::Left => Ok(EditorCommand::Move(Direction::Left)),
                KeyCode::Right => Ok(EditorCommand::Move(Direction::Right)),
                KeyCode::PageUp => Ok(EditorCommand::Move(Direction::PageUp)),
                KeyCode::PageDown => Ok(EditorCommand::Move(Direction::PageDown)),
                KeyCode::Home => Ok(EditorCommand::Move(Direction::Home)),
                KeyCode::End => Ok(EditorCommand::Move(Direction::End)),
                _ => Err(format!("Key code not supported: {code:?}")),
            },
            Event::Resize(width, height) => Ok(EditorCommand::Resize(Size {
                width: width as usize,
                height: height as usize,
            })),
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}
