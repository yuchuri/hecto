use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};

#[derive(Default)]
pub struct Editor;

impl Editor {
    pub fn run(&self) {
        terminal::enable_raw_mode().unwrap();
        loop {
            match event::read() {
                Ok(Event::Key(event)) => {
                    println!("{event:?}\r");
                    match event.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        _ => {}
                    }
                }
                Err(err) => eprintln!("Error: {err}\r"),
                _ => {}
            }
        }
        terminal::disable_raw_mode().unwrap();
    }
}
