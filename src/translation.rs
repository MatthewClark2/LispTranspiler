use crate::data::LispDatum;
use crate::ast::ASTNode::{Literal, Definition, Call};
use crate::ast::{ASTNode, ASTVisitor};
use std::collections::HashMap;

/// Struct used to handle the translation of a single compilation unit. Will need to be linked for
/// multi-file projects.
pub struct TranslationUnit {
    visitors: Vec<Box<dyn ASTVisitor<Vec<ASTNode>>>>,
    statements: Vec<ASTNode>,
    _gensym_index: u64,
}

impl TranslationUnit {
    fn new(visitors: Vec<Box<dyn ASTVisitor<Vec<ASTNode>>>>, statements: Vec<ASTNode>) -> Self {
        TranslationUnit { visitors, statements, _gensym_index: 0 }
    }

    pub fn from(statements: Vec<ASTNode>) {
        // visitors = vec!(CallExpansion::new());
    }

    fn gensym(&mut self) -> String {
        let x = self._gensym_index;
        self._gensym_index += 1;
        format!("__Translation_Unit_gensym_{}", x)
    }
}

struct CallExpansion<'a> {
    tu: &'a mut TranslationUnit,
}

impl <'a> CallExpansion<'a> {
    // TODO(matthew-c21): This is probably horrifyingly unoptimized code. Fix it later.
    fn expand(&mut self, node: ASTNode) -> Vec<ASTNode> {
        let mut expansion: Vec<ASTNode> = vec![];

        match node {
            Call(_, args) => {
                let mut new_args: Vec<ASTNode> = vec![];

                for arg in args {
                    match arg {
                        Call(a, b) => {
                            let symbol = self.tu.gensym();
                            new_args.push(Literal(LispDatum::Symbol(symbol.clone())));

                            // This can be safely unwrapped as expand should always generate at least one ASTNode (the input node).
                            let sub_expansion = self.expand(Call(a, b));
                            let (last, init) = sub_expansion.split_last().unwrap();
                            expansion.push(Definition(symbol, Box::new(last.clone())));
                            expansion.append(&mut init.to_vec());
                        }
                        _ => new_args.push(arg),
                    }
                }
            }

            _ => expansion.push(node),
        }

        expansion
    }
}

impl <'a> ASTVisitor<Vec<ASTNode>> for CallExpansion<'a> {
    fn visit_literal(&mut self, node: &LispDatum) -> Result<Vec<ASTNode>, String> {
        unimplemented!()
    }

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<Vec<ASTNode>, String> {
        unimplemented!()
    }

    fn visit_definition(&mut self, name: &String, value: &ASTNode) -> Result<Vec<ASTNode>, String> {
        unimplemented!()
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

// TODO(matthew-c21): Add reference to translation unit.
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

    pub fn visit_all(&mut self, ast: &Vec<ASTNode>) -> Result<String, String> {
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
    fn visit_literal(&mut self, node: &LispDatum) -> Result<String, String> {
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

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<String, String> {
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

    fn visit_definition(&mut self, name: &String, value: &ASTNode) -> Result<String, String> {
        let v = value.accept(self)?;
        Ok(format!("LispDatum** {} = {};", name, v))
    }
}