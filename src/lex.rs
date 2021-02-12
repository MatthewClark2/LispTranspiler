use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, digit0, digit1};
use nom::combinator::{map, opt, recognize, complete};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;
use std::fmt::Debug;
use std::str::FromStr;

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

    pub fn value(&self) -> TokenValue {
        self.value.clone()
    }

    pub fn line(&self) -> u32 {
        self.line.clone()
    }
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

pub fn start(_input: &str) -> Result<Vec<Token>, String> {
    unimplemented!()
}

// Auxiliary functions
fn is_token_terminal(ch: char) -> bool {
    ch.is_whitespace() || ch == '(' || ch == ')'
}

fn is_symbolic_start(ch: char) -> bool {
    vec![
        '*', '$', '+', '-', '_', '!', '?', '/', '%', '&', '^', '~', '<', '>', '=', '@',
    ]
    .contains(&ch)
        || ch.is_alphabetic()
}

fn is_symbolic_part(ch: char) -> bool {
    is_symbolic_start(ch) || ch.is_numeric()
}

fn signed<T>(
    f: &'static dyn Fn(&str) -> IResult<&str, T>,
    required: bool,
) -> Box<dyn Fn(&str) -> IResult<&str, T>>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    if required {
        Box::new(move |input| {
            map!(input, recognize!(pair!(sign, f)), |x| FromStr::from_str(x)
                .unwrap())
        })
    } else {
        Box::new(move |input| {
            map!(input, recognize!(pair!(signopt, f)), |x| FromStr::from_str(
                x
            )
            .unwrap())
        })
    }
}

// Base parsers
named!(sign <&str, &str>,
    alt!(tag!("+") | tag!("-"))
);

named!(signopt <&str, Option<&str>>,
    opt!(sign)
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
            tag_no_case!("e"),
            signopt,
            digit1
        )
    )
);

fn floating(input: &str) -> IResult<&str, f64> {
    map(
        recognize(tuple((digit1, opt(complete(preceded(tag("."), digit0))), opt(complete(exponent))))),
        |x| FromStr::from_str(x).unwrap(),
    )(input)
}

named!(digits <&str, i32>,
    map!(
        digit1,
        |x| FromStr::from_str(x).unwrap()
    )
);

named!(string_content <&str, &str>,
    alt!(escape | is_not!("\"\n\r\\"))
);

// Basic string escapes not including unicode. This function explicitly does not perform the escape
//  since the resultant string still needs to be copied into a C program.
fn escape(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        char('\\'),
        // `alt` tries each parser in sequence, returning the result of
        // the first successful match
        alt((
            // The `value` parser returns a fixed value (the first argument) if its
            // parser (the second argument) succeeds. In these cases, it looks for
            // the marker characters (n, r, t, etc) and returns the matching
            // character (\n, \r, \t, etc).
            tag("a"),
            tag("b"),
            tag("e"),
            tag("f"),
            tag("n"),
            tag("r"),
            tag("t"),
            tag("v"),
            tag("\\"),
            tag("\'"),
            tag("\""),
        )),
    ))(input)
}

// Main Parsers

fn int(input: &str) -> IResult<&str, TokenValue> {
    let r: (&str, i32) = signed(&digits, false)(input)?;

    Ok((r.0, TokenValue::Int(r.1)))
}

fn float(input: &str) -> IResult<&str, TokenValue> {
    let r: (&str, f64) = signed(&floating, false)(input)?;

    Ok((r.0, TokenValue::Float(r.1)))
}

// TODO(matthew-c21): Maybe check for zero division here.
fn rational(input: &str) -> IResult<&str, TokenValue> {
    let top = signed(&digits, false);
    let r = tuple!(input, top, tag!("/"), digits)?;

    let x = r.1;
    let num = x.0;
    let den = x.2;

    Ok((r.0, TokenValue::Rational(num, den)))
}

fn complex(input: &str) -> IResult<&str, TokenValue> {
    let re = signed(&floating, false);
    let im = signed(&floating, true);
    let r = tuple!(
        input,
        re,
        alt!(im | value!(1.0, tag!("+")) | value!(-1.0, tag!("-"))),
        tag!("i")
    )?;

    let x = r.1;
    let re = x.0;
    let im = x.1;

    Ok((r.0, TokenValue::Complex(re, im)))
}

fn keyword(input: &str) -> IResult<&str, TokenValue> {
    let r = pair!(input, tag!(":"), symbol_content)?;

    Ok((r.0, TokenValue::Keyword((r.1).1)))
}

fn symbol(input: &str) -> IResult<&str, TokenValue> {
    let r = symbol_content(input)?;

    Ok((r.0, TokenValue::Symbol(r.1)))
}

fn string(input: &str) -> IResult<&str, TokenValue> {
    // let stop_char: dyn Fn(&str) -> IResult<&str, &str> = one_of("\\\"\n\r");
    let r = delimited(tag("\""), many0(string_content), tag("\""))(input)?;

    let mut s: String = String::new();
    (r.1).iter().for_each(|x| s.push_str(x));

    Ok((r.0, TokenValue::Str(s)))
}

fn boolean(input: &str) -> IResult<&str, TokenValue> {
    let v = pair!(input, tag!("#"), alt!(tag!("t") | tag!("f")))?;

    match (v.1).1 {
        "t" => Ok((v.0, TokenValue::True)),
        "f" => Ok((v.0, TokenValue::False)),
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::lex::TokenValue::*;
    use crate::lex::*;

    #[test]
    fn valid_ints() {
        assert_eq!(int("123"), Ok(("", Int(123))));
        assert_eq!(int("+123 asdf"), Ok((" asdf", Int(123))));
        assert_eq!(int("-123 "), Ok((" ", Int(-123))));
    }

    #[test]
    fn valid_floats() {
        assert_eq!(float("4.0"), Ok(("", Float(4.0))));
        assert_eq!(float("-4. "), Ok((" ", Float(-4.0))));
        assert_eq!(float("4.0e1"), Ok(("", Float(4e1))));
        assert_eq!(float("4.e1"), Ok(("", Float(4.0e1))));
        assert_eq!(float("4.0E1"), Ok(("", Float(4.0e1))));
        assert_eq!(float("4.E1"), Ok(("", Float(4.0e1))));
        assert_eq!(float("4.0E+1"), Ok(("", Float(4.0e1))));
        assert_eq!(float("-4e-1"), Ok(("", Float(-4.0e-1))));
    }

    // NOTE(matthew-c21): This function also implicitly test keywords, as keywords use the same
    //  function. Since keywords and symbols are guaranteed to be syntactically similar, there's no
    //  reason not to use them in this manner.
    #[test]
    fn valid_symbol_content() {
        assert_eq!(symbol_content("sadf"), Ok(("", String::from("sadf"))));
        assert_eq!(symbol_content("?"), Ok(("", String::from("?"))));
        assert_eq!(symbol_content("+12a"), Ok(("", String::from("+12a"))));
    }

    #[test]
    fn valid_keyword_content() {
        assert_eq!(keyword(":1"), Ok(("", Keyword("1".to_string()))));
        assert_eq!(keyword(":sadf"), Ok(("", Keyword("sadf".to_string()))));
        assert_eq!(keyword(":?"), Ok(("", Keyword("?".to_string()))));
        assert_eq!(keyword(":+12a"), Ok(("", Keyword("+12a".to_string()))));
    }

    #[test]
    fn valid_complex() {
        assert_eq!(complex("-1.25+2e3i"), Ok(("", Complex(-1.25, 2e3))));
        assert_eq!(complex("1+i"), Ok(("", Complex(1.0, 1.0))));
        assert_eq!(complex("1.-i"), Ok(("", Complex(1.0, -1.0))));
    }

    #[test]
    fn valid_rational() {
        // TODO(matthew-c21): Comprehensive testing should ensure that things like +1/+2 get lexed
        //  as symbols.
        assert_eq!(rational("1/2"), Ok(("", Rational(1, 2))));
        assert_eq!(rational("-1/2"), Ok(("", Rational(-1, 2))));
        assert_eq!(rational("+1/2"), Ok(("", Rational(1, 2))));
    }

    #[test]
    fn exhaustive() {}

    #[test]
    fn booleans() {
        // TODO(matthew-c21): This means more comprehensive tests will need to fail to return bool
        //  tokens when there is a non-terminal character following a boolean expression.
        assert_eq!(boolean("#t"), Ok(("", True)));
        assert_eq!(boolean("#f"), Ok(("", False)));
        assert_eq!(boolean("#ft"), Ok(("t", False)));
    }

    #[test]
    fn strings() {
        // assert_eq!(string("hello, world"), Ok(("", Str("hello, world".to_string()))));
        assert_eq!(string("\"\""), Ok(("", Str("".to_string()))));
        assert_eq!(
            string("\"hello, world\""),
            Ok(("", Str("hello, world".to_string())))
        );
        assert_eq!(
            string("\"hello \\\" world\""),
            Ok(("", Str("hello \\\" world".to_string())))
        );
        assert_eq!(
            string("\"goodbye\\\"\""),
            Ok(("", Str("goodbye\\\"".to_string())))
        )
    }

    #[test]
    fn parens() {}
}
