use nom::bytes::complete::take_while;

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
named!(signopt <&str, i32>,
    map!(opt!(alt!(tag!("+") | tag!("-"))), to_sign)
);

named!(symbol_content<&str, String>,
    map!(
        take_while(is_symbolic_part),
        String::from
    )
);

/**
fn floating(input: &str) -> IResult<&str, f64> {}

// Main parsers
fn int(input: &str, line: i32) -> IResult<&str, Token> {}

fn float(input: &str, line: i32) -> IResult<&str, Token> {}

fn rational(input: &str, line: i32) -> IResult<&str, Token> {}

fn complex(input: &str, line: i32) -> IResult<&str, Token> {}

fn string(input: &str, line: i32) -> IResult<&str, Token> {}

fn keyword(input: &str, line: i32) -> IResult<&str, Token> {}

fn symbol(input: &str, line: i32) -> IResult<&str, Token> {}
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
