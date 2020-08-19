use crate::lex::IntLexHalt::{IntTerminal, Imaginary, Numerator, Decimal};
use crate::lex::Number::{Int, Float, Complex, Rational};
use crate::lex::FloatLexHalt::FloatTerminal;

// TODO(matthew-c21): Consider expanding with longs and arbitrary precision types.
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Number {
    Int(i32),
    Float(f64),
    Complex(f64, f64),

    // Rationals should be simplified at compile time, not lexing time.
    Rational(i32, i32),
}

#[derive(Debug)]
pub enum Token {
    Number(Number),
    String(String),
    Open,
    Close,
    Keyword(String),

    // For now, they can only contain letters.
    Symbol(String),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Number(x), Token::Number(y)) => x == y,
            (Token::String(x), Token::String(y)) => x == y,
            (Token::Open, Token::Open) => true,
            (Token::Close, Token::Close) => true,
            (Token::Keyword(x), Token::Keyword(y)) => x == y,
            (Token::Symbol(x), Token::Symbol(y)) => x == y,
            _ => false,
        }
    }
}

enum IntLexHalt { Decimal, Numerator, IntTerminal, Imaginary }

enum FloatLexHalt { Imaginary, FloatTerminal }

pub fn start(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::<Token>::new();

    let mut i = input;

    while !i.is_empty() {
        let c = i.chars().nth(0).unwrap();
        let r = match c {
            '(' => Ok((Token::Open, &i[1..])),
            ')' => Ok((Token::Close, &i[1..])),
            '"' => consume_string(&i[1..]),  // Skip the opening quote.
            ':' => consume_keyword(i),       // Skip the opening colon.
            x if is_space(x) => continue,
            x if is_numeric(x) || x == '.' => consume_number(i),
            x if is_alpha(x) => consume_symbol(i),
            _ => Err(format!("Unrecognized character `{}`", c)),
        }?;

        tokens.push(r.0);
        i = r.1;
    }

    Ok(tokens)
}

// TODO(matthew-c21): Rust probably has these built in better.
fn is_numeric(s: char) -> bool {
    ('0'..'9').contains(&s)
}

fn is_alpha(s: char) -> bool {
    ('a'..'z').contains(&s) || ('A'..'Z').contains(&s)
}

fn is_space(s: char) -> bool {
    [' ', '\t', '\n', '\r'].contains(&s)
}

fn is_terminal_symbol(s: char) -> bool {
    ['(', ')'].contains(&s) || is_space(s)
}

fn consume_string(input: &str) -> Result<(Token, &str), String> {
    let mut i = input;
    let mut s = String::new();

    while !i.is_empty() {
        match i.chars().nth(0).unwrap() {
            '"' => break,
            '\n'  => return Err("Unexpected newline while lexing string".to_string()),
            '\\' => {
                let r = consume_escape(&i[1..])?;
                s.push(r.0);
                i = r.1;
            },
            c => s.push(c),
        }

        i = &i[1..];
    }

    Ok((Token::String(s), i))
}

fn consume_escape(input: &str) -> Result<(char, &str), String> {
    let e = input.chars().nth(0);

    e.map_or(Err("Unexpected end of input.".to_string()), |c| {
        let c = match c {
            't' => '\t',
            'n' => '\n',
            'r' => '\r',
            _   => return Err(format!("Unexpected escape sequence `\\{}`", c).to_string())
        };

        Ok((c, &input[1..]))
    })
}

fn consume_keyword(input: &str) -> Result<(Token, &str), String> {
    Err(input.to_string())
}

fn consume_symbol(input: &str) -> Result<(Token, &str), String> {
    Err(input.to_string())
}

fn consume_number(input: &str) -> Result<(Token, &str), String> {
    // TODO(matthew-c21): This method fails for a+bi form complex numbers. Fix that.
    match consume_int(input) {
        Ok((x, out, IntTerminal)) => Ok((Token::Number(Int(x.parse().unwrap())), out)),
        Ok((x, out, Imaginary)) => Ok((Token::Number(Complex(0., x.parse().unwrap())), out)),
        Ok((x, out, Decimal)) => match continue_float(out, x) {
            Ok((x, out, FloatLexHalt::Imaginary)) => Ok((Token::Number(Complex(0., x.parse().unwrap())), out)),
            Ok((x, out, FloatTerminal)) => Ok((Token::Number(Float(x.parse().unwrap())), out)),
            Err(msg) => Err(msg),
        }
        Ok((x, out, Numerator)) => match consume_int(out) {
            Ok((y, out, IntTerminal)) => Ok((Token::Number(Rational(x.parse().unwrap(), y.parse().unwrap())), out)),
            Ok(_) => Err(String::from("Illegal continuation of rational number.")),
            Err(x) => Err(x),
        }
        Err(msg) => Err(msg)
    }
}

fn continue_float(input: &str, start: String) -> Result<(String, &str, FloatLexHalt), String> {
    // assert_eq!(input.chars().nth(0).unwrap(), '.', "Float must begin with decimal");

    let mut i = input;
    let mut s = start;
    s.push('.');

    while !i.is_empty() && !is_terminal_symbol(i.chars().nth(0).unwrap()){
        let x = i.chars().nth(0).unwrap();
        i = &i[1..];

        match x {
            'i' => return Ok((s, i, FloatLexHalt::Imaginary)),
            x if is_numeric(x) => s.push(x),
            x if is_terminal_symbol(x) => return Ok((s, i, FloatTerminal)),
            _ => return Err(format!("Unexpected value while lexing float `{}`", x))
        }
    }

    Ok((s, i, FloatTerminal))
}

fn consume_int(input: &str) -> Result<(String, &str, IntLexHalt), String> {
    let mut i = input;
    let mut s = String::new();

    while !i.is_empty() {
        let x = i.chars().nth(0).unwrap();
        i = &i[1..];

        match x {
            '.' => return Ok((s, i, Decimal)),
            '/' => return Ok((s, i, Numerator)),
            'i' => return Ok((s, i, Imaginary)),
            x if is_terminal_symbol(x) => return Ok((s, i, IntTerminal)),
            x if is_numeric(x) => s.push(x),
            x => return Err(format!("Expected part of number. Found `{}`", x))
        }
    }

    Ok((s, i, IntTerminal))
}

#[cfg(test)]
mod tests {
    use crate::lex::*;
    use crate::lex::start;

    #[test]
    fn lparen_only() {
    }

    #[test]
    fn rparen_only() {}

    #[test]
    #[ignore]
    fn valid_escaped_string() {}

    #[test]
    #[ignore]
    fn invalid_escaped_string() {}

    #[test]
    #[ignore]
    fn unicode_escaped_string() {}

    #[test]
    fn valid_numbers() {
        // let r = start("123 12i 6/12 1.23 0. .8 .83i 1.i").unwrap();
        let r1 = &start("123").unwrap()[0];
        let r2 = &start("12i").unwrap()[0];
        let r3 = &start("6/12").unwrap()[0];
        let r4 = &start("1.23").unwrap()[0];
        let r5 = &start("0.").unwrap()[0];
        let r6 = &start(".8").unwrap()[0];
        let r7 = &start(".83i").unwrap()[0];
        let r8 = &start("1.i").unwrap()[0];
        // assert_eq!(r.len(), 8);
        assert_eq!(*r1, Token::Number(Int(123)));
        assert_eq!(*r2, Token::Number(Complex(0., 12.)));
        assert_eq!(*r3, Token::Number(Rational(6, 12)));
        assert_eq!(*r4, Token::Number(Float(1.23)));
        assert_eq!(*r5, Token::Number(Float(0.)));
        assert_eq!(*r6, Token::Number(Float(0.8)));
        assert_eq!(*r7, Token::Number(Complex(0., 0.83)));
        assert_eq!(*r8, Token::Number(Complex(0., 1.)));
    }

    #[test]
    fn invalid_numbers() {}

    #[test]
    #[ignore]
    fn valid_symbols() {}

    #[test]
    #[ignore]
    fn invalid_symbols() {}

    #[test]
    #[ignore]
    fn valid_keywords() {}

    #[test]
    #[ignore]
    fn invalid_keywords() {}
}
