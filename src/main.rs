#[macro_use]
extern crate nom;

use std::{env, fs};

mod lex;
mod parse;

fn main() {
    // foo();
    let programs: Vec<String> = env::args().collect();
    // let programs = vec!["(format 3i)"];

    for program in &programs[1..] {
        let contents = fs::read_to_string(program).expect("Something went wrong reading the file");
        let x = run(contents.as_str()).unwrap();
        println!("{}", x);
    }
}

fn run(_program: &str) -> Result<String, String> {
    // let tokens = lex::start("(format (* 1 2 3))  (format 17i) (format 1.28) (format (+ 6 7 (* 2 7)))").unwrap();
    unimplemented!();
}
