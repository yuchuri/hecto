use std::io::{self, Read};

fn main() {
    for b in io::BufReader::new(io::stdin()).bytes() {
        let c = b.unwrap() as char;
        println!("{c}");
        if c == 'q' {
            break;
        }
    }
}
