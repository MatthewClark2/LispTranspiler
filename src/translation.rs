use crate::data::LispDatum;
use crate::parse::Statement;
use crate::ast::ASTNode::{Literal, Call};
use crate::ast::{ASTNode, ASTVisitor};

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
        for statement in ast {
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
                    // TODO(matthew-c21): Not every argument needs a comma.
                    out.push(',');
                });

                out.push(')');

                out
            },
            _ => unimplemented!()
        }
    }
}