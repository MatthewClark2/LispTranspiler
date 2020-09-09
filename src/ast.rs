use crate::data::LispDatum;
use crate::parse::Statement;
use crate::ast::ASTNode::{Literal, Call};

// NOTE(matthew-c21): Special forms to be added here.
pub enum ASTNode {
    Literal(LispDatum),
    Call(Box<ASTNode>, Vec<ASTNode>),
}

impl ASTNode {
    fn from_index(statements: Vec<Statement>, start: usize) -> Result<Vec<Self>, String> {
        let mut ast: Vec<Self> = Vec::new();

        for statement in &statements[start..] {
            match statement {
                Statement::Terminal(d) => ast.push(Literal(d.clone())),
                Statement::List(statements) => {
                    if statements.len() == 0 {
                        ast.push(Literal(LispDatum::Nil));
                    } else {
                        let stmts = statements;

                        match &statements[0] {
                            Statement::Terminal(LispDatum::Symbol(x)) => {
                                let args = ASTNode::from_index(stmts.to_vec(), 1)?;
                                ast.push(Call(Box::new(Literal(LispDatum::Symbol(x.clone()))), args))
                            }
                            _ => return Err("First element of list must be a symbol.".to_string())
                        }
                    }
                }
            }
        }

        Ok(ast)
    }

    pub fn from(statements: Vec<Statement>) -> Result<Vec<Self>, String> {
        ASTNode::from_index(statements, 0)
    }

    pub fn accept<T>(&self, visitor: &dyn ASTVisitor<T>) -> T {
        match self {
            ASTNode::Literal(d) => visitor.visit_literal(d),
            ASTNode::Call(c, a) => visitor.visit_call(c, a),
        }
    }
}

// NOTE(matthew-c21): This is subject to change in response to special forms.
pub trait ASTVisitor<T> {
    fn visit_literal(&self, node: &LispDatum) -> T;

    fn visit_call(&self, callee: &ASTNode, args: &Vec<ASTNode>) -> T;
}

// All optimizers should be in the form ASTNode -> ASTNode.

// TODO(matthew-c21): For now, everything is run straight from the main function. Later on, I'll
//  need to break it up to account for functions and (possibly) imports.
fn preamble() -> &'static str {
        "#include \"lisp.h\"\n\
        int main() {\n"
}

fn postamble() -> &'static str {
    "return 0;\n}"
}

pub struct TranspilationVisitor {}

impl TranspilationVisitor {
    pub fn new() -> Self {
        TranspilationVisitor {}
    }

    pub fn visit_all(&self, ast: &Vec<ASTNode>) -> String {
        let mut out = String::new();

        out.push_str(preamble());
        for (i, statement) in ast.iter().enumerate() {
            out.push_str(statement.accept::<String>(self).as_str());
            out.push_str(";\n");
        }
        out.push_str(postamble());

        out
    }
}

impl ASTVisitor<String> for TranspilationVisitor {
    fn visit_literal(&self, node: &LispDatum) -> String {
        match node {
            LispDatum::Cons(_, _) => unimplemented!(),
            LispDatum::Complex(r, i) => format!("new_complex({},{})", r, i),
            LispDatum::Real(x) => format!("new_real({})", x),
            LispDatum::Rational(p, q) => format!("new_rational({},{})", p, q),
            LispDatum::Integer(i) => format!("new_integer({})", i),
            LispDatum::Symbol(s) => format!("new_symbol({})", s),
            LispDatum::Nil => format!("get_nil()"),
        }
    }

    fn visit_call(&self, callee: &ASTNode, args: &Vec<ASTNode>) -> String {
        match callee {
            Literal(LispDatum::Symbol(s)) => {
                // TODO(matthew-c21): Add function lookup.
                let mut out = format!("{}(", s);

                args.iter().for_each(|arg| {
                    out.push_str(arg.accept::<String>(self).as_str());
                    out.push(',');
                });

                out.push(')');

                out
            },
            _ => unimplemented!()
        }
    }
}
