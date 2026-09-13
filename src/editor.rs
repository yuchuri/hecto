use std::io::Result;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};

#[derive(Default)]
pub struct Editor {}

impl Editor {
    pub fn run(&self) {
        if let Err(err) = self.repl() {
            panic!("{err:#?}");
        }
        print!("Goodbye.\r\n")
    }

    fn repl(&self) -> Result<()> {
        terminal::enable_raw_mode()?;
        loop {
            if let Event::Key(event) = event::read()? {
                println!("{event:?}\r");
                if let KeyCode::Char('q') = event.code {
                    break;
                }
            }
        }
        terminal::disable_raw_mode()
    }
}
