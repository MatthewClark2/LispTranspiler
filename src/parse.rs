use crate::lex::Token;
use crate::data::LispDatum;
use crate::data::LispDatum::Nil;

pub struct ParseTree {
    body: Vec<Box<dyn Statement>>,
}

pub trait StatementVisitor {
    fn visit_terminal(&mut self, terminal: &Terminal);

    fn visit_call(&mut self, call: &Call);
}

pub trait Statement {
    fn accept(self, visitor: &mut dyn StatementVisitor);
}

pub struct Terminal {
    value: LispDatum
}

impl Statement for Terminal {
    fn accept(self, visitor: &mut dyn StatementVisitor) {
        visitor.visit_terminal(&self);
    }
}

pub struct Call {
    callee: LispDatum,
    args: Vec<LispDatum>,
}

impl Statement for Call {
    fn accept(self, visitor: &mut dyn StatementVisitor) {
        visitor.visit_call(&self);
    }
}

fn statement(tokens: &[Token]) -> Result<(&[Token], Box<dyn Statement>), String> {
    if tokens.is_empty() {
        return Err(String::from("Nothing to parse."));
    }

    match &tokens[0] {
        Token::Open => {
            // Start of a call.
            call(&tokens[..])
        }
        Token::Close => {
            Err(String::from("Unexpected end of list."))
        }
        // Numbers, strings, symbols, etc.
        &x => Ok((&tokens[1..], Box::new(Terminal { value: LispDatum::from_token(x) })))
    }
}

fn call(tokens: &[Token]) -> Result<(&[Token], Box<dyn Statement>), String> {
    let mut args: Vec<LispDatum> = Vec::new();

    if tokens.is_empty() {
        return Err(String::from("No call to parse."));
    } else if tokens[0] != Token::Open {
        return Err(String::from("Expected call to start with paren."));
    }

    let mut x = &tokens[1..];
    let (x, callee) = statement(x)?;

    while !x.is_empty() {
        if x[0] != Token::Close {
            return if args.len() == 0 {
                Ok((&x[1..], Box::new(Terminal { value: Nil })))
            } else {
                Ok((&x[1..], Call { callee, args }))
            };
        }
        let (x, arg) = statement(x)?;
        args.push(arg)
    }

    Err(String::from("Expected end of list."))
}

fn terminal(tokens: &[Token]) -> Result<(&[Token], Box<dyn Statement>), String> {}

impl ParseTree {
    pub fn from(tokens: Vec<Token>) -> Result<Self, String> {
        Ok(ParseTree { body: Vec::new() })
    }
}


