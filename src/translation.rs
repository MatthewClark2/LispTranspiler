use crate::data::LispDatum;
use crate::ast::ASTNode::{Literal};
use crate::ast::{ASTNode, ASTVisitor};
use std::collections::HashMap;
use std::ops::Deref;

/// Struct used to handle the translation of a single compilation unit. Will need to be linked for
/// multi-file projects.
pub struct TranslationUnit {
    visitors: Vec<dyn ASTVisitor<Vec<ASTNode>>>,
    statements: Vec<ASTNode>,
}

impl TranslationUnit {
    fn new(visitors: Vec<dyn ASTVisitor<Vec<ASTNode>>>, statements: Vec<ASTNode>) -> Self {
        TranslationUnit { visitors, statements }
    }

    pub fn from(statements: Vec<ASTNode>) {
        visitors = vec!(CallExpansion::new());
    }
}

struct CallExpansion {}

impl CallExpansion {
    fn expand(node: ASTNode) -> Vec<ASTNode> {
        let mut defs: Vec<ASTNode> = vec![];
        let mut expansion: Vec<ASTNode> = vec![];

        match node {
            Literal(_) => expansion.push(node),
            ASTNode::Call(_, args) => {
                for arg in args {
                    match arg {
                        Literal(_) => expansion.push(arg),
                        ASTNode::Call(_, _) => {}
                    }
                }
            }
        }

        expansion
    }
}

impl ASTVisitor<Vec<ASTNode>> for CallExpansion {
    fn visit_literal(&self, node: &LispDatum) -> Result<Vec<ASTNode>, String> {
        Ok(vec!(Literal(node.clone())))
    }

    fn visit_call(&self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<Vec<ASTNode>, String> {
        /*
        Get list of nested calls in order from innermost to outermost.
        For each call,

        Try doing it one at a time - i.e. unroll the innermost function call in a line so that
         */

        let mut arg_defs: Vec<ASTNode> = vec!();

        self.recursive_visit_call(callee, args, arg_defs)?;
        // TODO(matthew-c21): Need some kind of recursive algorithm to get to innermost function and evaluate outwards. Replace any non-terminal argument.

        Err(format!("Fuck"))
    }
}

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

    pub fn visit_all(&self, ast: &Vec<ASTNode>) -> Result<String, String> {
        let mut out = String::new();

        out.push_str(preamble());
        for statement in ast {
            out.push_str(statement.accept::<String>(self)?.as_str());
            out.push_str(";\n");
        }
        out.push_str(postamble());

        Ok(out)
    }
}

impl ASTVisitor<String> for TranspilationVisitor {
    fn visit_literal(&self, node: &LispDatum) -> Result<String, String> {
        let mut out: String = (self.generators)(node);

        out.push_str(match node {
            LispDatum::Complex(r, i) => format!("({},{})", r, i),
            LispDatum::Real(x) => format!("({})", x),
            LispDatum::Rational(p, q) => format!("({},{})", p, q),
            LispDatum::Integer(i) => format!("({})", i),
            LispDatum::Symbol(s) => format!("({})", s),
            LispDatum::Nil => format!("()"),
        }.as_str());

        Ok(out)
    }

    fn visit_call(&self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<String, String> {
        match callee {
            Literal(LispDatum::Symbol(s)) => {
                let mut out = format!("{}(", self.functions.get(s).unwrap());

                if args.len() > 0 {
                    out.push_str(args[0].accept::<String>(self)?.as_str());

                    for arg in &args[1..] {
                        out.push(',');
                        out.push_str(arg.accept::<String>(self)?.as_str());
                    }
                }

                out.push(')');

                Ok(out)
            }
            _ => Err(format!("No option for dynamic calls yet."))
        }
    }
}