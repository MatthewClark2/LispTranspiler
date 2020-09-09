use crate::data::LispDatum;
use crate::ast::ASTNode::{Literal};
use crate::ast::{ASTNode, ASTVisitor};
use std::collections::HashMap;

// TODO(matthew-c21): For now, everything is run straight from the main function. Later on, I'll
//  need to break it up to account for functions and (possibly) imports.
fn preamble() -> &'static str {
    "#include \"lisp.h\"\n\
        int main() {\n"
}

fn postamble() -> &'static str {
    "return 0;\n}"
}

fn default_generators(d: &LispDatum) -> String {
    String::from(match d {
        LispDatum::Complex(_, _) => "new_complex",
        LispDatum::Real(_) => "new_real",
        LispDatum::Rational(_, _) => "new_rational",
        LispDatum::Integer(_) => "new_integer",
        LispDatum::Symbol(_) => "new_symbol",
        LispDatum::Nil => "get_nil()",
    })
}

pub struct TranspilationVisitor {
    functions: HashMap<String, String>,
    generators: &'static dyn Fn(&LispDatum) -> String,
}

impl TranspilationVisitor {
    pub fn new() -> Self {
        TranspilationVisitor {
            functions: [
                ("*", "multiply"),
                ("+", "add"),
                ("-", "subtract"),
                ("/", "divide"),
                ("mod", "mod"),
                ("division", "division"),
                ("format", "display"),
                ("eqv", "eqv")
            ].iter().map(|pair| (String::from(pair.0), String::from(pair.1))).clone().collect(),
            generators: &default_generators,
        }
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
        let mut out: String = (self.generators)(node);

        out.push_str(match node {
            LispDatum::Complex(r, i) => format!("({},{})", r, i),
            LispDatum::Real(x) => format!("({})", x),
            LispDatum::Rational(p, q) => format!("({},{})", p, q),
            LispDatum::Integer(i) => format!("({})", i),
            LispDatum::Symbol(s) => format!("({})", s),
            LispDatum::Nil => format!("()"),
        }.as_str());

        out
    }

    fn visit_call(&self, callee: &ASTNode, args: &Vec<ASTNode>) -> String {
        match callee {
            Literal(LispDatum::Symbol(s)) => {
                let mut out = format!("{}(", self.functions.get(s).unwrap());

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