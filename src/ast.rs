use crate::lex::Token;
use std::collections::{HashSet, HashMap};
use crate::parse::ParseTree;
use crate::ast::Value::*;
use crate::ast::Statement::*;
use crate::lex::TokenValue::Symbol;
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
                    Err((start, String::from("Empty lists are unsupported as syntax elements.")))
                }

                match &elems[0] {
                    ParseTree::Leaf(Token { line, value: Symbol(s) }) if &s[..] == "if" => {
                        if elems.len() != 4 {
                            Err((line, String::from(format!("Expected exactly 3 arguments in `if` special form. Found {}.", elems.len()))))
                        }
                    }
                    ParseTree::Leaf(Token { line, value: Symbol(s) }) if &s[..] == "define" => {
                        if elems.len() != 3 {
                            Err((line, String::from(format!("Expected exactly 2 arguments in `define` special form. Found {}.", elems.len()))))
                        }
                    }
                    ParseTree::Leaf(t) => {
                        match &t {
                            Token { value: Symbol(s), .. } => {
                                let mut args = Vec::new();

                                for subtree in &elems[1..] {
                                    match Self::try_from(subtree). {
                                        Ok(ASTNode::Value(v)) => args.push(v),

                                    }
                                }
                                let args: Vec<Result<Self, Self::Error>> = &elems[1..].iter().map(Self::try_from::<ParseTree>).collect();
                                Ok(ASTNode::Value(Call(Box::new(Literal(t.clone())), args)))
                            },
                            _ => Err(String::from("Symbols are the only literal value that may be invoked.")),
                        }
                    }
                    ParseTree::Branch(elems, start, _stop) => {}
                }
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
    Definition(String, Value, Scope, bool),
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

/* AST Construction

1. Split off special forms via series of visitors. Each should ensure that the special form is
   correctly formed.
   - The SymbolTable should start being built when splitting off Definitions.
2. Unfurl nested function calls.
3. Trace through the program to ensure that symbols are properly utilized.
4. Change any re-defined variables to ensure that they refer to the correct value in the scope in
   which they have been redefined.

 */