use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::terminal::Size;

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
                KeyCode::Up => Ok(Move::Up),
                KeyCode::Down => Ok(Move::Down),
                KeyCode::Left => Ok(Move::Left),
                KeyCode::Right => Ok(Move::Right),
                KeyCode::PageUp => Ok(Move::PageUp),
                KeyCode::PageDown => Ok(Move::PageDown),
                KeyCode::Home => Ok(Move::Home),
                KeyCode::End => Ok(Move::End),
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
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Edit::Insert(c)),
            (KeyCode::Tab, KeyModifiers::NONE) => Ok(Edit::Insert('\t')),
            (KeyCode::Enter, KeyModifiers::NONE) => Ok(Edit::InsertNewLine),
            (KeyCode::Backspace, KeyModifiers::NONE) => Ok(Edit::DeleteBackward),
            (KeyCode::Delete, KeyModifiers::NONE) => Ok(Edit::Delete),
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
}

impl TryFrom<KeyEvent> for System {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::CONTROL {
            match code {
                KeyCode::Char('q') => Ok(System::Quit),
                KeyCode::Char('s') => Ok(System::Save),
                _ => Err(format!("Unsupported CONTROL+{code:?} combination")),
            }
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
