#[macro_use]
extern crate nom;

use crate::ast::*;
use crate::parse::ParseTree;
use std::{env, fs};

mod ast;
mod lex;
mod parse;
// mod ast;

fn main() {
    // foo();
    let programs: Vec<String> = env::args().collect();
    // let programs = vec!["(format 3i)"];

    for program in &programs[1..] {
        let contents = fs::read_to_string(program).expect("Something went wrong reading the file");
        println!("{:#?}", run(contents.as_str()));
    }
}

fn run(program: &str) -> Result<Vec<ASTNode>, (u32, String)> {
    // let tokens = lex::start("(format (* 1 2 3))  (format 17i) (format 1.28) (format (+ 6 7 (* 2 7)))").unwrap();
    let tokens = lex::start(program).unwrap();
    let parse_tree = parse::parse(&tokens).unwrap();

    let mut output = String::new();

    for tree in parse_tree.as_slice() {
        output.push_str(tree.to_pretty_string().as_str());
        output.push('\n');
        output.push('\n');
    }

    println!("{}", output);

    let cnde = ConditionUnroll;
    let fne = FunctionUnfurl;
    let mut sym_table = SymbolTable::dummy();
    let ast = ast::construct_ast(&parse_tree)?;

    let mut output = Vec::new();

    for line in ast {
        let first_expansion = cnde.try_visit(&line, &mut sym_table)?;

        for line in first_expansion {
            output.append(&mut fne.try_visit(&line, &mut sym_table)?);
        }
    }

    Ok(output)
}
