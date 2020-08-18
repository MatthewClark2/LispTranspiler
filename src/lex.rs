use crate::lex::IntLexHalt::{IntTerminal, Imaginary, Numerator, Decimal};
use crate::lex::Number::{Int, Float, Complex, Rational};
use crate::lex::FloatLexHalt::FloatTerminal;

// TODO(matthew-c21): Consider expanding with longs and arbitrary precision types.
pub enum Number {
    Int(i32),
    Float(f64),
    Complex(f64, f64),
    Rational(i32, i32),
}

pub enum Token {
    Number(Number),
    String(String),
    Open,
    Close,
    Keyword(String),

    // For now, they can only contain letters.
    Symbol(String),
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
            x if is_numeric(x) => consume_number(i),
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
    assert_eq!(input.chars().nth(0).unwrap(), '.', "Float must begin with decimal");

    let mut i = &input[1..];
    let mut s = start;
    s.push('.');

    while !i.is_empty() {
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

    Err(String::from("Integer lexing not interrupted."))
}

#[cfg(test)]
mod tests {
    use crate::lex::*;
    use crate::lex::start;

    #[test]
    fn lparen_only() {
        let r = start("(").expect("unable to lex");
        
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
    fn valid_numbers() {}

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
