use std::io::Result;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn run(&mut self) {
        if let Err(err) = self.repl() {
            panic!("{err:#?}");
        }
        print!("Goodbye.\r\n")
    }

    fn repl(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        loop {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }) = event::read()?
            {
                println!(
                    "Code: {code:?} Modifiers: {modifiers:?} Kind: {kind:?} State: {state:?} \r"
                );
                match code {
                    KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => {
                        self.should_quit = true
                    }
                    _ => (),
                }
                if self.should_quit {
                    break;
                }
            }
        }
        terminal::disable_raw_mode()
    }
}
