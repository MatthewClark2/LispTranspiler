use std::str::FromStr;
use nom::bytes::complete::take_while;
use nom::character::complete::{digit0, digit1};
use nom::IResult;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    // This technically limits file sizes, but the maximum size is tremendous.
    line: u32,
    value: TokenValue,
}

impl Token {
    /// Quick factory function primarily used for testing. Try to avoid this for real code.
    pub fn from(value: TokenValue) -> Self {
        Token { line: 0, value }
    }

    pub fn value(&self) -> TokenValue { self.value.clone() }

    pub fn line(&self) -> u32 { self.line.clone() }
}

// TODO(matthew-c21): Add macro symbols ('), and other special symbols (#', .)
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Int(i32),
    Float(f64),
    Complex(f64, f64),
    Rational(i32, i32),
    // TODO(matthew-c21): Add unicode escaped strings.
    Str(String),
    Keyword(String),
    Symbol(String),
    Open,
    Close,
    True,
    False,
}

pub fn start(_input: &str) -> Vec<Token> {
    unimplemented!()
}

// Auxiliary functions
fn is_token_terminal(ch: char) -> bool {
    ch.is_whitespace() || ch == '(' || ch == ')'
}

fn is_symbolic_start(ch: char) -> bool {
    vec!('*', '$', '+', '-', '_', '!', '?', '/', '%', '&', '^', '~', '<', '>', '=', '@').contains(&ch) || ch.is_alphabetic()
}

fn is_symbolic_part(ch: char) -> bool {
    is_symbolic_start(ch) || ch.is_numeric()
}

fn to_sign(sign: Option<&str>) -> i32 {
    match sign {
        Some("-") => -1,
        _ => 1
    }
}

// Base parsers
named!(signopt <&str, Option<&str>>,
    opt!(alt!(tag!("+") | tag!("-")))
);

named!(symbol_content<&str, String>,
    map!(
        take_while(is_symbolic_part),
        String::from
    )
);

named!(exponent<&str, &str>,
    recognize!(
        tuple!(
            alt!(tag!("e") | tag!("E")),
            signopt,
            digit1
        )
    )
);

named!(floating <&str, f64>,
    map!(
        recognize!(
            tuple!(
                digit1,
                tag!("."),
                digit0,
                opt!(exponent)
            )
        ),
        |x| { FromStr::from_str(x).unwrap() }
    )
);

named!(digits <&str, i32>,
    map!(
        digit1,
        |x| FromStr::from_str(x).unwrap()
    )
);

fn signed<T>(f: &'static dyn Fn(&str) -> IResult<&str, T>) -> Box<dyn Fn(&str) -> IResult<&str, T>> {
    Box::new(|x| {
        map!(
            x,
            pair!(map!(signopt, to_sign), f),
            |y| { y.0 * y.1 }
        )
    })
}

// Main Parsers

fn int(input: &str, line: u32) -> IResult<&str, Token> {
    let r: (&str, i32) = map!(
        input,
        recognize!(
            pair!(signopt, digit1)
        ),
        |x| { FromStr::from_str(x).unwrap() }
    )?;

    Ok((r.0, Token { value: TokenValue::Int(r.1), line }))
}


fn float(input: &str, line: u32) -> IResult<&str, Token> {
    let r: (&str, f64) = map!(
        input,
        recognize!(
            pair!(signopt, floating)
        ),
        |x| { FromStr::from_str(x).unwrap() }
    )?;

    Ok((r.0, Token { value: TokenValue::Float(r.1), line }))
}


fn rational(input: &str, line: u32) -> IResult<&str, Token> {
    let r = tuple!(input, signed<i32>, tag!("/"), digits)?;

    let num = (r.1).0 * (r.1).1;
    let den = r.3;

    Ok((r.0, Token { line, value: TokenValue::Rational(num, den)}))
}

/**

// Main parsers



fn complex(input: &str, line: u32) -> IResult<&str, Token> {}

fn string(input: &str, line: u32) -> IResult<&str, Token> {}

fn keyword(input: &str, line: u32) -> IResult<&str, Token> {}

fn symbol(input: &str, line: u32) -> IResult<&str, Token> {}
*/
#[cfg(test)]
mod tests {
    use crate::lex::*;

    #[test]
    fn valid_ints() {}

    #[test]
    fn invalid_ints() {}

    #[test]
    fn valid_floats() {}

    #[test]
    fn invalid_floats() {}

    #[test]
    fn valid_symbols() {
        assert_eq!(symbol_content("sadf"), Ok(("", String::from("sadf"))));
        assert_eq!(symbol_content("?"), Ok(("", String::from("?"))));
        assert_eq!(symbol_content("+12a"), Ok(("", String::from("+12a"))));
    }

    #[test]
    fn invalid_symbols() {}

    #[test]
    fn valid_keywords() {}

    #[test]
    fn invalid_keywords() {}

    #[test]
    fn valid_complex() {}

    #[test]
    fn invalid_complex() {}

    #[test]
    fn valid_rational() {}

    #[test]
    fn invalid_rational() {}

    #[test]
    fn exhaustive() {}

    #[test]
    fn booleans() {}

    #[test]
    fn parens() {}
}
