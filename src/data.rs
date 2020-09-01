use crate::lex::{Token, Number};
use crate::data::LispDatum::{Integer, Rational};

pub enum LispDatum {
    Cons(Box<LispDatum>, Box<LispDatum>),
    Complex(f64, f64),
    Real(f64),
    Rational(i32, i32),
    Integer(i32),
    Symbol(String),
    Nil,
}

impl LispDatum {
    pub fn from_token(token: Token) -> Self {
        match token {
            Token::Number(n) => {
                match n {
                    Number::Int(i) => LispDatum::Integer(i),
                    Number::Float(f) => LispDatum::Real(f),
                    Number::Complex(a, b) => LispDatum::Complex(a, b),
                    Number::Rational(a, b) => LispDatum::Rational(a, b),
                }
            },
            Token::Symbol(s) => LispDatum::Symbol(s),
            _ => unimplemented!(),
        }
    }
}