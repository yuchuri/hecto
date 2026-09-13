use std::io::{self, Read};

use crossterm::terminal;

fn main() {
    terminal::enable_raw_mode().unwrap();
    for b in io::BufReader::new(io::stdin()).bytes() {
        let c = b.unwrap() as char;
        println!("{c}\r");
        if c == 'q' {
            terminal::disable_raw_mode().unwrap();
            break;
        }
    }
}
