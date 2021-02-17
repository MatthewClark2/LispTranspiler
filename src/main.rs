#[macro_use]
extern crate nom;

use std::{env, fs};
use std::any::Any;
use crate::parse::ParseTree;

mod lex;
mod parse;
// mod ast;

fn main() {
    // foo();
    let programs: Vec<String> = env::args().collect();
    // let programs = vec!["(format 3i)"];

    for program in &programs[1..] {
        let contents = fs::read_to_string(program).expect("Something went wrong reading the file");
        run(contents.as_str());
    }
}

fn run(program: &str) -> () {
    // let tokens = lex::start("(format (* 1 2 3))  (format 17i) (format 1.28) (format (+ 6 7 (* 2 7)))").unwrap();
    let tokens = lex::start(program).unwrap();
    let parse_tree = parse::parse(&tokens).unwrap();

    let mut output = String::new();

    for tree in parse_tree {
        output.push_str(tree.to_pretty_string().as_str());
        output.push('\n');
        output.push('\n');
    }

    println!("{}", output)
}
