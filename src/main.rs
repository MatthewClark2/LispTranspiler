#[macro_use]
extern crate nom;

use crate::ast::*;
use std::{env, fs};

mod ast;
mod lex;
mod parse;
mod transpile;

fn main() {
    // foo();
    let programs: Vec<String> = env::args().collect();
    // let programs = vec!["(format 3i)"];

    for program in &programs[1..] {
        let contents = fs::read_to_string(program).expect("Something went wrong reading the file");
        println!("{}", run(contents.as_str()));
    }
}

fn examine(program: &str) -> Result<(), (u32, String)> {
    println!("########## Initial Program ##########\n{}", program);

    // let tokens = lex::start("(format (* 1 2 3))  (format 17i) (format 1.28) (format (+ 6 7 (* 2 7)))").unwrap();
    let tokens = lex::start(program).unwrap();
    let parse_tree = parse::parse(&tokens).unwrap();

    let mut output = String::new();

    for tree in parse_tree.as_slice() {
        output.push_str(format!("{:?}", parse_tree).as_str());
        output.push('\n');
        output.push('\n');
    }

    println!("\n########## Parse Tree ##########\n{}", output);

    let cnde = ConditionUnroll;
    let fne = FunctionUnfurl;
    let sv = SymbolValidation;
    let mut sym_table = SymbolTable::dummy();
    let ast = ast::construct_ast(&parse_tree)?;

    println!("\n########## Initial AST ##########\n{:#?}", ast);

    let mut preliminary_pass = Vec::new();

    for line in ast {
        preliminary_pass.append(&mut cnde.try_visit(&line, &mut sym_table)?);
    }

    println!(
        "\n########## AST After Conditional Unroll ##########\n{:#?}",
        preliminary_pass
    );

    let mut secondary_pass = Vec::new();

    for line in preliminary_pass {
        secondary_pass.append(&mut fne.try_visit(&line, &mut sym_table)?);
    }

    println!(
        "\n########## AST After Function Unrolling ##########\n{:#?}",
        secondary_pass
    );

    let mut tertiary_pass = Vec::new();

    for line in secondary_pass {
        tertiary_pass.push(sv.try_visit(&line, &mut sym_table)?);
    }

    println!("\n########## Final AST ##########\n{:#?}", tertiary_pass);

    Ok(())
}

fn run(program: &str) -> String {
    let mut sym_table = SymbolTable::load(None);
    let sv = SymbolValidation;
    let fne = FunctionUnfurl;
    let ce = ConditionUnroll;

    let tokens = lex::start(program).unwrap();
    let parse_tree = parse::parse(&tokens).unwrap();
    let ast = ast::construct_ast(&parse_tree).unwrap();

    let ast: Vec<ASTNode> = ast
        .iter()
        .map(|n| ce.visit(n, &mut sym_table))
        .flatten()
        .collect();
    let ast: Vec<ASTNode> = ast
        .iter()
        .map(|n| fne.visit(n, &mut sym_table))
        .flatten()
        .collect();
    let ast = ast.iter().map(|n| sv.visit(n, &mut sym_table)).collect();

    let mut transpiler = transpile::Transpiler::new(sym_table);
    transpiler.translate(ast)
}
