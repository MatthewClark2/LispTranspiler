use crate::ast::TranspilationVisitor;

mod lex;
mod parse;
mod data;
mod ast;

fn main() {
    let tokens = lex::start("(* 1 2 3)").unwrap();
    let parse_tree = parse::parse(tokens).unwrap();
    let ast = ast::ASTNode::from(parse_tree).unwrap();
    let mut visitor = TranspilationVisitor::new();

    for node in ast {
        node.accept(&mut visitor);
    }

    println!("{}", visitor);
}
