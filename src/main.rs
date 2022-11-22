extern crate synacor_challenge;

use self::synacor_challenge::*;
use std::env;

fn main() {
    match env::args().nth(1) {
        None => println!("Please provide an input path."),
        Some(path) => {
            let mut machine: Machine = Machine::new();
            let read = machine.load(&path).unwrap_or(0);
            println!("Read {} bytes, executing.", read);
            println!("=========================");
            machine.exec();
        }
    }
}
