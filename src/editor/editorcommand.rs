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
    Insert(char),
    Resize(Size),
    Backspace,
    Delete,
    Enter,
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
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    Ok(Self::Insert(ch))
                }
                (KeyCode::Tab, _) => Ok(Self::Insert('\t')),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::End, _) => Ok(Self::Move(Direction::End)),
                (KeyCode::Backspace, _) => Ok(EditorCommand::Backspace),
                (KeyCode::Delete, _) => Ok(EditorCommand::Delete),
                (KeyCode::Enter, _) => Ok(EditorCommand::Enter),
                _ => Err(format!("Key code not supported: {code:?}")),
            },
            Event::Resize(width, height) => Ok(Self::Resize(Size {
                width: width as usize,
                height: height as usize,
            })),
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}
