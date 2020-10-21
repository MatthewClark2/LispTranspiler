use std::{env, fs};

use crate::translation::TranslationUnit;

mod lex;
mod parse;
mod data;
mod ast;
mod translation;

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

fn foo() {
    let tokens = lex::start("(define -x 1)").unwrap();
    let parse_tree = parse::parse(tokens).unwrap();
    let ast = ast::ASTNode::from(parse_tree).unwrap();
    let tu = TranslationUnit::from(ast).unwrap();
    print!("{}", tu.translate().unwrap())
}

fn run(program: &str) -> Result<String, String> {
    // let tokens = lex::start("(format (* 1 2 3))  (format 17i) (format 1.28) (format (+ 6 7 (* 2 7)))").unwrap();
    let tokens = lex::start(program)?;
    let parse_tree = parse::parse(tokens)?;
    let ast = ast::ASTNode::from(parse_tree)?;
    let tu = TranslationUnit::from(ast)?;
    tu.translate()
}
