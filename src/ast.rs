use crate::data::LispDatum;
use crate::parse::Statement;
use crate::ast::ASTNode::{Literal, Call};

// NOTE(matthew-c21): Special forms to be added here.
#[derive(Clone)]
pub enum ASTNode {
    Literal(LispDatum),
    Call(Box<ASTNode>, Vec<ASTNode>),
    Definition(String, Box<ASTNode>),
    Condition(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
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

    pub fn accept<T>(&self, visitor: &mut dyn ASTVisitor<T>) -> Result<T, String> {
        match self {
            ASTNode::Literal(d) => visitor.visit_literal(d),
            ASTNode::Call(c, a) => visitor.visit_call(c, a),
            ASTNode::Definition(n, v) => visitor.visit_definition(n, v),
            ASTNode::Condition(x, y, z) => visitor.visit_condition(x, y, z),
        }
    }
}

// NOTE(matthew-c21): This is subject to change in response to special forms.
// TODO(matthew-c21): Add some kind of Error handling.
pub trait ASTVisitor<T> {
    fn visit_literal(&mut self, node: &LispDatum) -> Result<T, String>;

    fn visit_call(&mut self, callee: &ASTNode, args: &Vec<ASTNode>) -> Result<T, String>;

    fn visit_definition(&mut self, name: &String, value: &ASTNode) -> Result<T, String>;

    fn visit_condition(&mut self, cond: &ASTNode, if_true: &ASTNode, if_false: &ASTNode) -> Result<T, String>;
}

// All optimizers should be in the form ASTNode -> ASTNode.
