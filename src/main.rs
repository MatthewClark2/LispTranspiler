mod lex;
mod parse;
mod data;
mod ast;
mod translation;

fn main() {
    let tokens = lex::start("(format (* 1 2 3))  (format 17i) (format 1.28) (format (+ 6 7 (* 2 7)))").unwrap();
    let parse_tree = parse::parse(tokens).unwrap();
    let ast = ast::ASTNode::from(parse_tree).unwrap();
}
