mod lex;
mod parse;
mod data;
mod ast;
mod translation;

fn main() {
    let tokens = lex::start("(* 1 2 3)").unwrap();
    let parse_tree = parse::parse(tokens).unwrap();
    let ast = ast::ASTNode::from(parse_tree).unwrap();
    let visitor = translation::TranspilationVisitor::new();

    println!("{}", visitor.visit_all(&ast));
}
