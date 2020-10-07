use crate::data::LispDatum;
use crate::ast::ASTNode::{Literal, Definition, Call};
use crate::ast::{ASTNode, ASTVisitor};
use std::collections::HashMap;
use crate::data::LispDatum::Symbol;

#[derive(Copy, Clone)]
struct Gensym {
    index: u64,
}

impl Gensym {
    fn inc(&mut self) -> u64 {
        let x = self.index;
        self.index += 1;
        x
    }

    fn gensym(&mut self, base: &str) -> String {
        format!("__{}_gensym_{}", base, self.inc())
    }

    fn new(index: u64) -> Self {
        Gensym { index }
    }
}

/// Struct used to handle the translation of a single compilation unit. Will need to be linked for
/// multi-file projects.
pub struct TranslationUnit {
    statements: Vec<ASTNode>,
    _gensym: Gensym,
}

impl TranslationUnit {
    pub fn from(statements: Vec<ASTNode>) -> TranslationUnit {
        let mut me = TranslationUnit { statements, _gensym: Gensym::new(0) };

        // Create each listener iteratively
        me.apply(Box::new(CallExpansion::new(&me)));

        me
    }

    fn apply(&mut self, mut visitor: Box<dyn ASTVisitor<Vec<ASTNode>>>) -> Option<String> {
        let mut statements: Vec<ASTNode> = vec![];

        for statement in &self.statements {
            match statement.accept(visitor.as_mut()) {
                Ok(mut x) => statements.append(&mut x),
                Err(msg) => return Some(msg),
            }
        }

        self.statements = statements;

        None
    }

    pub fn translate(&self) -> Result<String, String> {
        let mut output = String::from(preamble());

        let mut translation_visitor = TranspilationVisitor::new(self._gensym.index);

        for statement in &self.statements {
            let x: Result<String, String> = statement.accept(&mut translation_visitor);

            match x {
                Ok(out) => output.push_str(out.as_str()),
                Err(msg) => return Err(msg),
            }
        }

        output.push_str(postamble());

        Ok(output)
    }
}

struct CallExpansion {
    gensym: Gensym,
}

impl CallExpansion {
    fn new(tu: &TranslationUnit) -> Self {
        CallExpansion { gensym: tu._gensym.clone() }
    }
}

impl ASTVisitor<Vec<ASTNode>> for CallExpansion {
    fn visit_literal(&mut self, node: &LispDatum) -> Result<Vec<ASTNode>, String> {
        Ok(vec![Literal(node.clone())])
    }

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<Vec<ASTNode>, String> {
        let mut expansion: Vec<ASTNode> = vec![];

        let mut updated_args: Vec<ASTNode> = vec![];

        for arg in args {
            match &arg {
                Call(_, _) => {
                    let sub_expansion: Result<Vec<ASTNode>, String> = arg.accept(self);

                    match sub_expansion {
                        Ok(sub_expansion) => {
                            let symbol = self.gensym.gensym("CallExpansion");
                            updated_args.push(Literal(Symbol(symbol.clone())));

                            let x = sub_expansion.split_last();
                            match x {
                                Some((last, init)) => {
                                    expansion.append(&mut init.to_vec());
                                    expansion.push(Definition(symbol, Box::new(last.clone())));
                                }
                                None => panic!("Empty subcall.")
                            }
                        }
                        Err(msg) => {
                            return Err(msg);
                        }
                    }
                }
                _ => updated_args.push(arg.clone()),
            }
        }

        expansion.push(Call(Box::new(callee.clone()), updated_args));

        Ok(expansion)
    }

    fn visit_definition(&mut self, name: &String, value: &ASTNode) -> Result<Vec<ASTNode>, String> {
        Ok(vec![Definition(name.clone(), Box::new(value.clone()))])
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
        LispDatum::Nil => "get_nil",
    })
}

// TODO(matthew-c21): Add reference to translation unit.
struct TranspilationVisitor {
    functions: HashMap<String, String>,
    generators: &'static dyn Fn(&LispDatum) -> String,
    gensym: Gensym,
}

impl TranspilationVisitor {
    pub fn new(index: u64) -> Self {
        TranspilationVisitor {
            functions: [
                ("*", "multiply"),
                ("+", "add"),
                ("-", "subtract"),
                ("/", "divide"),
                ("mod", "mod"),
                ("division", "division"),
                ("format", "format"),
                ("eqv", "eqv")
            ].iter().map(|pair| (String::from(pair.0), String::from(pair.1))).clone().collect(),
            generators: &default_generators,
            gensym: Gensym::new(index),
        }
    }

    /// Splits a call into its expansion and the actual valued call. Used for defining variables as the value returned from a call.
    fn split_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<(String, String), String> {
        let mut output = String::new();

        match callee {
            Literal(Symbol(s)) if self.functions.contains_key(s) => {
                let symbol = self.gensym.gensym("ArgumentCollection");
                output.push_str(format!("struct LispDatum* {}[{}];\n", symbol, args.len()).as_str());

                for (i, arg) in args.iter().enumerate() {
                    output.push_str(format!("{}[{}] = {};\n", symbol, i, arg.accept(self)?).as_str());
                }

                Ok((output, format!("{}({}, {});\n", self.functions.get(s).unwrap(), symbol, args.len())))
            }
            _ => Err("Callee is not a built in function and may not be invoked at this time.".to_string()),
        }
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
            LispDatum::Symbol(s) => {
                // For now, assume that symbols cannot be generated at runtime.
                return Ok(format!("{}", s))
            },
            LispDatum::Nil => format!("()"),
        }.as_str());

        Ok(out)
    }

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<String, String> {
        let x = self.split_call(callee, args)?;
        let mut y = x.0;
        y.push_str(x.1.as_str());
        Ok(y)
    }

    fn visit_definition(&mut self, name: &String, value: &ASTNode) -> Result<String, String> {
        match value {
            Call(callee, args) => {
                let mut output = String::new();
                let (pre, value) = self.split_call(callee, args)?;

                output.push_str(pre.as_str());
                output.push_str(format!("struct LispDatum* {} = {};", name, value).as_str());
                Ok(output)
            }
            Literal(Symbol(s)) => Ok(format!("struct LispDatum* {} = {};", name, s)),
            Literal(d) => Ok(format!("struct LispDatum* {} = {};", name, ((self.generators)(d)))),
            Definition(_, _) => Err("Cannot assign to a definition.".to_string()),
        }
    }
}