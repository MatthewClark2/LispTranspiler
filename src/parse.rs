use crate::data::LispDatum;
use crate::lex::Token;
use crate::parse::Statement::{Terminal, List};
use crate::data::LispDatum::{Symbol, Integer, Real, Complex, Rational};

pub enum Statement {
    Terminal(LispDatum),
    List(Vec<Statement>),
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, String> {
    let mut statements: Vec<Statement> = Vec::new();
    let mut t = &tokens[..];

    while !t.is_empty() {
        let r = statement(t)?;
        t = r.1;
        statements.push(r.0);
    }

    Ok(statements)
}

// The following auxiliary functions expect at least one token to be present.
fn statement(tokens: &[Token]) -> Result<(Statement, &[Token]), String> {
    let rest = &tokens[1..];

    match &tokens[0] {
        &Token::Int(i) => Ok((Terminal(Integer(i)), rest)),
        &Token::Float(f) => Ok((Terminal(Real(f)), rest)),
        &Token::Complex(r, i) => Ok((Terminal(Complex(r, i)), rest)),
        &Token::Rational(n, d) => Ok((Terminal(Rational(n, d)), rest)),
        &Token::Str(_) => unimplemented!(),
        &Token::Open => {
            list(tokens)
        }
        &Token::Close => return Err("Unexpected end of list.".to_string()),
        &Token::Keyword(_) => unimplemented!(),
        Token::Symbol(s) => Ok((Terminal(Symbol(s.clone())), rest)),
    }
}

fn list(tokens: &[Token]) -> Result<(Statement, &[Token]), String> {
    let mut vals: Vec<Statement> = Vec::new();
    let mut t = &tokens[..];

    while !t.is_empty() {
        let r = statement(t)?;
        t = r.1;
        vals.push(r.0);
    }

    Ok((List(vals), t))
}
