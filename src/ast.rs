use crate::ast::Statement::*;
use crate::ast::Value::*;
use crate::lex::Token;
use crate::lex::TokenValue::Symbol;
use crate::parse::ParseTree;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

#[derive(Clone)]
pub enum ASTNode {
    Value(Value),
    Statement(Statement),
}

pub fn construct_ast(_parsed: &Vec<ParseTree>) -> Result<Vec<ASTNode>, (u32, String)> {
    unimplemented!()
}

impl From<Token> for ASTNode {
    fn from(t: Token) -> Self {
        Self::Value(Literal(t.clone()))
    }
}

impl TryFrom<&ParseTree> for ASTNode {
    type Error = (u32, String);

    fn try_from(tree: &ParseTree) -> Result<Self, Self::Error> {
        match &tree {
            ParseTree::Leaf(t) => Ok(Self::from(t.clone())),
            ParseTree::Branch(elems, start, _stop) => {
                if elems.len() == 0 {
                    return Err((
                        *start,
                        String::from("Empty lists are unsupported as syntax elements."),
                    ));
                }

                return match &elems[0] {
                    ParseTree::Leaf(Token {
                        line,
                        value: Symbol(s),
                    }) if &s[..] == "if" => {
                        if elems.len() != 4 {
                            return Err((
                                *line,
                                String::from(format!(
                                    "Expected exactly 3 arguments in `if` special form. Found {}.",
                                    elems.len()
                                )),
                            ));
                        }

                        let cond = Self::try_from(&elems[1])?;
                        let if_true = Self::try_from(&elems[2])?;
                        let if_false = Self::try_from(&elems[3])?;

                        match (cond, if_true, if_false) {
                            (ASTNode::Value(a), ASTNode::Value(b), ASTNode::Value(c)) => Ok(ASTNode::Value(Value::Condition(Box::new(a.clone()), Box::new(b.clone()), Box::new(c.clone())))),
                            _ => Err((*line, String::from("Expected values for condition, true, and false branches of condition.")))
                        }
                    }
                    ParseTree::Leaf(Token {
                        line,
                        value: Symbol(s),
                    }) if &s[..] == "define" => {
                        if elems.len() != 3 {
                            return Err((*line, String::from(format!("Expected exactly 2 arguments in `define` special form. Found {}.", elems.len()))));
                        }

                        let defined = Self::try_from(&elems[1])?;
                        let value = Self::try_from(&elems[2])?;

                        match (defined, value) {
                            (
                                ASTNode::Value(Literal(Token {
                                    value: Symbol(s), ..
                                })),
                                ASTNode::Value(v),
                            ) => Ok(ASTNode::Statement(Definition(s.clone(), v.clone()))),
                            (
                                ASTNode::Value(Literal(Token {
                                    value: Symbol(s),
                                    line,
                                })),
                                _,
                            ) => Err((line, String::from("Can only assign a symbol to a value."))),
                            (_, ASTNode::Value(v)) => {
                                Err((*line, String::from("Can only assign a value to a symbol.")))
                            }
                            _ => Err((*line, String::from("Invalid definition."))),
                        }
                    }
                    ParseTree::Leaf(t) => match &t {
                        Token {
                            value: Symbol(s),
                            line,
                        } => {
                            let mut args = Vec::new();

                            for subtree in &elems[1..] {
                                match Self::try_from(subtree)? {
                                    (ASTNode::Value(v)) => args.push(v),
                                    _ => {
                                        return Err((
                                            *line,
                                            String::from("All arguments in call should be values."),
                                        ))
                                    }
                                }
                            }
                            let args: Vec<Result<Self, Self::Error>> =
                                (&elems[1..]).iter().map(Self::try_from).collect();

                            let mut values = Vec::new();
                            for arg in args {
                                match arg {
                                        Ok(ASTNode::Value(v)) => (values.push(v.clone())),
                                        Ok(_) => return Err((*line, stringify!("Expected a value to be passed as an argument. Found: {}.", &elems[0]).to_string())),
                                        _ => return arg,
                                    }
                            }
                            Ok(ASTNode::Value(Call(Box::new(Literal(t.clone())), values)))
                        }
                        _ => Err((
                            t.line(),
                            String::from("Symbols are the only literal value that may be invoked."),
                        )),
                    },
                    ParseTree::Branch(elems, start, _stop) => {
                        // TODO(matthew-c21): Later, it should be possible to invoke lambda special
                        //  forms as well as functions that may return functions.
                        Err((
                            *start,
                            String::from("Compound forms cannot be used as function calls."),
                        ))
                    }
                };
            }
        }
    }
}

#[derive(Clone)]
pub enum Value {
    // Should only hold valued tokens. Anything else should be removed during the parsing step.
    Literal(Token),

    // obviously callee and arguments
    Call(Box<Value>, Vec<Value>),

    // condition, value if true, value if false
    Condition(Box<Value>, Box<Value>, Box<Value>),
}

#[derive(Clone)]
pub enum Statement {
    // name, value, scope, is_redefinition
    // Definition(String, Value, Scope, bool),
    Definition(String, Value),
}

pub trait ASTVisitor<T> {
    fn visit(ast: &ASTNode) -> T;
}

#[derive(Clone)]
struct SymbolTable {
    natives: HashSet<String>,
    defs: HashMap<String, SymbolTableEntry>,
}

#[derive(Clone)]
struct SymbolTableEntry {
    c_name: String,
    scopes: Vec<Scope>,
}

#[derive(Clone)]
pub enum Scope {
    /// File global scoping. Available in all subsequent code, as well as in previously defined functions.
    Global,

    /// Local variable, such as that in a `let` expression. Gets an associated tag to match it with.
    Local(String),

    /// Function parameters.
    Function(String),
}

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lex::start;
    use crate::parse::parse;

    fn force_from(input: &str) -> Vec<ASTNode> {
        parse(&start(input).unwrap())
            .unwrap()
            .iter()
            .map(ASTNode::try_from)
            .map(Result::unwrap)
            .collect()
    }

    #[test]
    fn from_literals() {
        let ast = force_from("hello");
        assert_eq!(1, ast.len());

        if let ASTNode::Value(Literal(t)) = &ast[0] {
            assert_eq!(Symbol(String::from("hello")), t.value())
        }
    }

    #[test]
    fn from_define() {}

    #[test]
    fn from_malformed_defines() {}

    #[test]
    fn from_condition() {}

    #[test]
    fn from_malformed_condition() {}

    #[test]
    // TODO(matthew-c21): After adding all the relevant listeners, replace this with something else.
    fn basic_comprehensive() {}

    /*
    Stil need:

    symbol table tracing and relevant errors
    validation of called functions
    function call unfurling
    conditional unfurling
     */
}

/* AST Construction

1. Split off special forms via series of visitors. Each should ensure that the special form is
   correctly formed.
   - The SymbolTable should start being built when splitting off Definitions.
2. Unfurl nested function calls.
3. Trace through the program to ensure that symbols are properly utilized.
4. Change any re-defined variables to ensure that they refer to the correct value in the scope in
   which they have been redefined.

 */
