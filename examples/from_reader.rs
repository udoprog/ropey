extern crate ropey;

use std::fs::File;
use std::io;
use std::io::Read;

use ropey::Rope;

fn main() {
    // Get filepath from commandline
    let filepath = if std::env::args().count() > 1 {
        std::env::args().nth(1).unwrap()
    } else {
        println!(
            "You must pass a filepath!  Only recieved {} arguments.",
            std::env::args().count()
        );
        panic!()
    };

    // Build rope from file contents
    let rope = Rope::from_reader(&mut io::BufReader::new(File::open(&filepath).unwrap())).unwrap();

    // Read the text into a string as well
    let text = {
        let mut input = String::new();
        let mut f = io::BufReader::new(File::open(&filepath).unwrap());
        f.read_to_string(&mut input).unwrap();
        input
    };

    // Verify that the rope and string match
    let mut idx = 0;
    for chunk in rope.chunks() {
        assert_eq!(chunk, &text[idx..(idx + chunk.len())]);
        idx += chunk.len();
    }
}
