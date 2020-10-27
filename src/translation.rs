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
    pub fn from(statements: Vec<ASTNode>) -> Result<TranslationUnit, String> {
        let mut me = TranslationUnit { statements, _gensym: Gensym::new(0) };


        // TODO(matthew-c21): Do all the name mangling at once, rather than doing just defs first
        me.apply(Box::new(UserDefinition::new()))?;
        me.apply(Box::new(Mangler::new(Gensym::new(0))))?;
        me.apply(Box::new(CallExpansion::new(Gensym::new(1024))))?;

        Ok(me)
    }

    fn apply(&mut self, mut visitor: Box<dyn ASTVisitor<Vec<ASTNode>>>) -> Result<(), String> {
        let mut statements: Vec<ASTNode> = vec![];

        for statement in &self.statements {
            statements.append(&mut statement.accept(visitor.as_mut())?);
        }

        self.statements = statements;

        Ok(())
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
    fn new(gensym: Gensym) -> Self {
        CallExpansion { gensym }
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
        LispDatum::Bool(true) => "get_true",
        LispDatum::Bool(false) => "get_false",
        LispDatum::Complex(_, _) => "new_complex",
        LispDatum::Real(_) => "new_real",
        LispDatum::Rational(_, _) => "new_rational",
        LispDatum::Integer(_) => "new_integer",
        LispDatum::Symbol(_) => "new_symbol",
        LispDatum::String(_) => "new_string",
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
            LispDatum::Bool(_) => format!("()"),
            LispDatum::Complex(r, i) => format!("({},{})", r, i),
            LispDatum::Real(x) => format!("({})", x),
            LispDatum::Rational(p, q) => format!("({},{})", p, q),
            LispDatum::Integer(i) => format!("({})", i),
            LispDatum::Symbol(s) => {
                // For now, assume that symbols cannot be generated at runtime.
                return Ok(format!("{}", s));
            }
            LispDatum::Nil => format!("()"),
            LispDatum::String(s) => format!("(\"{}\")", s),
        }.as_str());

        Ok(out)
    }

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<String, String> {
        let x = self.split_call(callee, args)?;
        let mut y = x.0;
        y.push_str(x.1.as_str());
        Ok(y)
    }

    // TODO(matthew-c21): A lot of extraneous code is generated, making it similar to a define then
    //  call.
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
            Literal(d) => Ok(format!("struct LispDatum* {} = {};", name, self.visit_literal(d)?)),
            Definition(_, _) => Err("Cannot assign to a definition.".to_string()),
        }
    }
}

// TODO(matthew-c21): Check for redefinitions of both user and native variables.
struct UserDefinition {}

impl UserDefinition {
    fn is_define(callee: &ASTNode) -> bool {
        match callee {
            Literal(Symbol(x)) if x == "define" => true,
            _ => false,
        }
    }

    fn new() -> Self {
        Self {}
    }
}

// TODO(matthew-c21): Name mangling needs to be pervasive (i.e. every symbol in the program gets
//  similarly mangled.
impl ASTVisitor<Vec<ASTNode>> for UserDefinition {
    fn visit_literal(&mut self, node: &LispDatum) -> Result<Vec<ASTNode>, String> {
        Ok(vec!(Literal(node.clone())))
    }

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<Vec<ASTNode>, String> {
        return if UserDefinition::is_define(callee) {
            // A definition takes two arguments: a symbol and a value.
            if 2 != args.len() {
                return Err("define special form takes two arguments.".to_string());
            }

            match &args[0] {
                Literal(Symbol(x)) => {
                    match &args[1] {
                        Literal(_) => {
                            Ok(vec!(Definition(x.clone(), Box::new(args[1].clone()))))
                        }
                        Call(callee, _) if !Self::is_define(callee.as_ref()) => {
                            Ok(vec!(Definition(x.clone(), Box::new(args[1].clone()))))
                        }
                        _ => Err("Cannot assign to that which has no value.".to_string())
                    }
                }
                _ => Err("First argument to define must be a symbol".to_string())
            }
        } else {
            Ok(vec!(Call(Box::from(callee.clone()), args.clone())))
        };
    }

    fn visit_definition(&mut self, name: &String, value: &ASTNode) -> Result<Vec<ASTNode>, String> {
        Ok(vec!(Definition(name.clone(), Box::from(value.clone()))))
    }
}

struct Mangler {
    gensym: Gensym,
    mangled_names: HashMap<String, String>,
}

impl Mangler {
    fn new(gensym: Gensym) -> Self {
        Self { gensym, mangled_names: HashMap::new() }
    }

    fn c_ify(&mut self, symbol: &String) -> String {
        if self.mangled_names.contains_key(symbol) {
            return self.mangled_names.get(symbol).unwrap().clone();
        }

        let mut result = String::from("user_");

        for ch in symbol.chars() {
            result.push_str((match ch {
                '+' => "_plus_".to_string(),
                '-' => "_dash_".to_string(),
                '*' => "_star_".to_string(),
                '/' => "_fshlash_".to_string(),
                '<' => "_lt_".to_string(),
                '>' => "_gt_".to_string(),
                '?' => "_question_".to_string(),
                '@' => "_at_".to_string(),
                '!' => "_exclaim_".to_string(),
                '=' => "_equals_".to_string(),
                _ => ch.to_string(),
            }).as_str());
        }

        let generated = self.gensym.gensym(&result);

        self.mangled_names.insert(symbol.clone(), generated.clone());

        generated
    }
}

impl ASTVisitor<Vec<ASTNode>> for Mangler {
    fn visit_literal(&mut self, node: &LispDatum) -> Result<Vec<ASTNode>, String> {
        match node {
            Symbol(x) => Ok(vec!(Literal(Symbol(self.c_ify(x))))),
            _ => Ok(vec!(Literal(node.clone())))
        }
    }

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<Vec<ASTNode>, String> {
        // TODO(matthew-c21): Mangle callee when user defined functions exist.

        // Visit each argument, attempting to mangle names along the way.
        let mut new_args: Vec<ASTNode> = Vec::new();

        for arg in args {
            let mut out = arg.accept::<Vec<ASTNode>>(self)?;

            if out.len() != 1 {
                return Err("Argument incorrectly expanded during name mangling.".to_string());
            }

            new_args.append(&mut out);
        }

        Ok(vec!(Call(Box::new(callee.clone()), new_args)))
    }

    fn visit_definition(&mut self, name: &String, value: &ASTNode) -> Result<Vec<ASTNode>, String> {
        let new_value: Vec<ASTNode> = value.accept(self)?;

        if new_value.len() != 1 {
            Err("Incorrect number of values generated by definition during mangling.".to_string())
        } else {
            // Cloning here is a hack since `new_value` is destroyed anyway. However, it's
            // non-trivial to move out of a Vec, assuming it's possible at all.
            Ok(vec!(Definition(self.c_ify(name).clone(), Box::new(new_value[0].clone()))))
        }
    }
}