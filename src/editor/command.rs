use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::size::Size;

#[derive(Clone, Copy)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

impl TryFrom<KeyEvent> for Move {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::NONE {
            match code {
                KeyCode::Up => Ok(Self::Up),
                KeyCode::Down => Ok(Self::Down),
                KeyCode::Left => Ok(Self::Left),
                KeyCode::Right => Ok(Self::Right),
                KeyCode::PageUp => Ok(Self::PageUp),
                KeyCode::PageDown => Ok(Self::PageDown),
                KeyCode::Home => Ok(Self::Home),
                KeyCode::End => Ok(Self::End),
                _ => Err(format!("Unsupported code: {code:?}")),
            }
        } else {
            Err(format!(
                "Unsupported key code: {code:?} or modifiers: {modifiers:?}"
            ))
        }
    }
}

#[derive(Clone, Copy)]
pub enum Edit {
    Insert(char),
    InsertNewLine,
    Delete,
    DeleteBackward,
}

impl TryFrom<KeyEvent> for Edit {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        match (code, modifiers) {
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Insert(c)),
            (KeyCode::Tab, KeyModifiers::NONE) => Ok(Self::Insert('\t')),
            (KeyCode::Enter, KeyModifiers::NONE) => Ok(Self::InsertNewLine),
            (KeyCode::Backspace, KeyModifiers::NONE) => Ok(Self::DeleteBackward),
            (KeyCode::Delete, KeyModifiers::NONE) => Ok(Self::Delete),
            _ => Err(format!(
                "Unsupported key code: {code:?} or modifiers: {modifiers:?}"
            )),
        }
    }
}

#[derive(Clone, Copy)]
pub enum System {
    Save,
    Resize(Size),
    Quit,
    Dismiss,
}

impl TryFrom<KeyEvent> for System {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::CONTROL {
            match code {
                KeyCode::Char('q') => Ok(Self::Quit),
                KeyCode::Char('s') => Ok(Self::Save),
                _ => Err(format!("Unsupported CONTROL+{code:?} combination")),
            }
        } else if modifiers == KeyModifiers::NONE && matches!(code, KeyCode::Esc) {
            Ok(Self::Dismiss)
        } else {
            Err(format!(
                "Unsupported key code: {code:?} or modifiers: {modifiers:?}"
            ))
        }
    }
}

#[derive(Clone, Copy)]
pub enum Command {
    Move(Move),
    Edit(Edit),
    System(System),
}

impl TryFrom<Event> for Command {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(key_event) => Edit::try_from(key_event)
                .map(Command::Edit)
                .or_else(|_| Move::try_from(key_event).map(Command::Move))
                .or_else(|_| System::try_from(key_event).map(Command::System))
                .map_err(|_| format!("Event not supported: {key_event:?}")),
            Event::Resize(width, height) => Ok(Self::System(System::Resize(Size {
                width: width as usize,
                height: height as usize,
            }))),
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}
