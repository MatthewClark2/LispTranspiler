use crate::lex::IntLexHalt::{IntTerminal, Imaginary, Numerator, Decimal};
use crate::lex::FloatLexHalt::FloatTerminal;
use crate::lex::Token::{Symbol, Int, Float, Complex, Rational, Str, Open, Close, Keyword};

// TODO(matthew-c21): Consider expanding with longs and arbitrary precision types.
// TODO(matthew-c21): Add lexical information (index in file, len, etc.).
// TODO(matthew-c21): Change this to a Literal(LispDatum) | SyntaxElement(s) enum.
#[derive(Debug)]
#[derive(Clone)]
pub enum Token {
    Int(i32),
    Float(f64),
    Complex(f64, f64),
    // Rationals should be simplified at compile time, not lexing time.
    Rational(i32, i32),
    Str(String),
    Open,
    Close,
    Keyword(String),

    // For now, they can only contain letters.
    Symbol(String),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Int(a), Int(b)) => a == b,
            (Float(x), Float(y)) => x == y,
            (Complex(a, b), Complex(c, d)) => a == c && b == d,
            // Rationals should be simplified at compile time, not lexing time.
            (Rational(a, b), Rational(c, d)) => a == c && b == d,
            (Str(x), Str(y)) => x == y,
            (Open, Open) => true,
            (Close, Close) => true,
            (Keyword(x), Keyword(y)) => x == y,
            (Symbol(x), Symbol(y)) => x == y,
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
            '(' => Ok((Open, &i[1..])),
            ')' => Ok((Close, &i[1..])),
            '"' => consume_string(&i[1..]),  // Skip the opening quote.
            ':' => consume_keyword(i),       // Skip the opening colon.
            x if is_space(x) => {
                i = &i[1..];
                continue;
            }
            x if is_numeric(x) || x == '.' => consume_number(i),
            x if is_symbol_start(x) => consume_symbol(i),
            _ => Err(format!("Unrecognized character `{}`", c)),
        }?;

        tokens.push(r.0);
        i = r.1;
    }

    Ok(tokens)
}

// TODO(matthew-c21): Rust probably has these built in better.
// TODO(matthew-c21): All of these create a slice each time they are called. This could probably be
//  changed to static allocation.
fn is_numeric(s: char) -> bool {
    ('0'..='9').contains(&s)
}

fn is_alpha(s: char) -> bool {
    ('a'..='z').contains(&s) || ('A'..='Z').contains(&s)
}

fn is_symbol_start(s: char) -> bool {
    is_alpha(s) || vec!('+', '-', '*', '/', '<', '>', '?', '@', '!', '_', '=').contains(&s)
}

fn is_symbol_part(s: char) -> bool {
    is_symbol_start(s) || is_numeric(s)
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
            '\n' => return Err("Unexpected newline while lexing string".to_string()),
            '\\' => {
                let r = consume_escape(&i[1..])?;
                s.push(r.0);
                i = r.1;
            }
            c => s.push(c),
        }

        i = &i[1..];
    }

    Ok((Str(s), i))
}

fn consume_escape(input: &str) -> Result<(char, &str), String> {
    let e = input.chars().nth(0);

    e.map_or(Err("Unexpected end of input.".to_string()), |c| {
        let c = match c {
            't' => '\t',
            'n' => '\n',
            'r' => '\r',
            _ => return Err(format!("Unexpected escape sequence `\\{}`", c).to_string())
        };

        Ok((c, &input[1..]))
    })
}

fn consume_keyword(input: &str) -> Result<(Token, &str), String> {
    Err(input.to_string())
}

fn consume_symbol(input: &str) -> Result<(Token, &str), String> {
    assert_ne!(input.len(), 0);
    let first = input.chars().nth(0).unwrap();
    assert!(is_symbol_start(first));

    let mut i = &input[1..];

    let mut sym = String::new();
    sym.push(first);

    while !i.is_empty() {
        let x = i.chars().nth(0).unwrap();

        if is_symbol_part(x) {
            sym.push(x)
        } else if is_terminal_symbol(x) {
            break;
        } else {
            return Err(format!("Unexpected `{}` while parsing symbol.", x));
        }

        i = &i[1..];
    }

    Ok((Symbol(sym), i))
}

fn consume_number(input: &str) -> Result<(Token, &str), String> {
    // TODO(matthew-c21): This method fails for a+bi form complex numbers. Fix that.
    match consume_int(input) {
        Ok((x, out, IntTerminal)) => Ok((Int(x.parse().unwrap()), out)),
        Ok((x, out, Imaginary)) => Ok((Complex(0., x.parse().unwrap()), out)),
        Ok((x, out, Decimal)) => match continue_float(out, x) {
            Ok((x, out, FloatLexHalt::Imaginary)) => Ok((Complex(0., x.parse().unwrap()), out)),
            Ok((x, out, FloatTerminal)) => Ok((Float(x.parse().unwrap()), out)),
            Err(msg) => Err(msg),
        }
        Ok((x, out, Numerator)) => match consume_int(out) {
            Ok((y, out, IntTerminal)) => Ok((Rational(x.parse().unwrap(), y.parse().unwrap()), out)),
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

    while !i.is_empty() && !is_terminal_symbol(i.chars().nth(0).unwrap()) {
        let x = i.chars().nth(0).unwrap();
        let rest = &i[1..];

        match x {
            'i' => return Ok((s, rest, FloatLexHalt::Imaginary)),
            x if is_numeric(x) => s.push(x),
            x if is_terminal_symbol(x) => return Ok((s, i, FloatTerminal)),
            _ => return Err(format!("Unexpected value while lexing float `{}`", x))
        }

        i = rest;
    }

    Ok((s, i, FloatTerminal))
}

fn consume_int(input: &str) -> Result<(String, &str, IntLexHalt), String> {
    let mut i = input;
    let mut s = String::new();

    while !i.is_empty() {
        let x = i.chars().nth(0).unwrap();
        let rest = &i[1..];

        match x {
            '.' => return Ok((s, rest, Decimal)),
            '/' => return Ok((s, rest, Numerator)),
            'i' => return Ok((s, rest, Imaginary)),
            x if is_terminal_symbol(x) => return Ok((s, i, IntTerminal)),
            x if is_numeric(x) => s.push(x),
            x => return Err(format!("Expected part of number. Found `{}`", x))
        }

        i = rest;
    }

    Ok((s, i, IntTerminal))
}

#[cfg(test)]
mod tests {
    use crate::lex::*;
    use crate::lex::start;

    #[test]
    fn lparen_only() {}

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
        assert_eq!(*r1, (Int(123)));
        assert_eq!(*r2, (Complex(0., 12.)));
        assert_eq!(*r3, (Rational(6, 12)));
        assert_eq!(*r4, (Float(1.23)));
        assert_eq!(*r5, (Float(0.)));
        assert_eq!(*r6, (Float(0.8)));
        assert_eq!(*r7, (Complex(0., 0.83)));
        assert_eq!(*r8, (Complex(0., 1.)));
    }

    #[test]
    fn invalid_numbers() {}

    #[test]
    fn valid_symbols() {
        let r = start("+").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], Symbol(String::from("+")));

        let r = start("_z").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], Symbol(String::from("_z")));

        let r = start("abc").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], Symbol(String::from("abc")));

        let r = start("++localhost++").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], Symbol(String::from("++localhost++")));

        let r = start("lispy-writing").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], Symbol(String::from("lispy-writing")));

        let r = start("+sNaKe_CaSe-").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], Symbol(String::from("+sNaKe_CaSe-")));
    }

    #[test]
    fn invalid_symbols() {}

    #[test]
    #[ignore]
    fn valid_keywords() {}

    #[test]
    #[ignore]
    fn invalid_keywords() {}

    #[test]
    fn function_call() {
        let r = start("(+ a b c+d*e 12. .2i)").unwrap();

        assert_eq!(r.len(), 8);
        assert_eq!(r[0], Open);
        assert_eq!(r[1], Symbol(String::from("+")));
        assert_eq!(r[2], Symbol(String::from("a")));
        assert_eq!(r[3], Symbol(String::from("b")));
        assert_eq!(r[4], Symbol(String::from("c+d*e")));
        assert_eq!(r[5], (Float(12.0)));
        assert_eq!(r[6], (Complex(0.0, 0.2)));
        assert_eq!(r[7], Close);
    }

    #[test]
    #[ignore]
    fn unbalanced_input() {}

    #[test]
    fn consecutive_tokens() {
        // I think the issue is that terminal symbols are being ignored on return.
        let r = start("(a) (1)) (c d)").unwrap();
        assert_eq!(r.len(), 11);

        assert_eq!(r[0], Open);
        assert_eq!(r[1], Symbol("a".to_string()));
        assert_eq!(r[2], Close);
        assert_eq!(r[3], Open);
        assert_eq!(r[4], Int(1));
        assert_eq!(r[5], Close);
        assert_eq!(r[6], Close);
        assert_eq!(r[7], Open);
        assert_eq!(r[8], Symbol("c".to_string()));
        assert_eq!(r[9], Symbol("d".to_string()));
        assert_eq!(r[10], Close);
    }

    #[test]
    fn multiple_statements() {
        let r = start("a 12 d1- (* 1i 2. (+ x 3)) ()").unwrap();

        assert_eq!(r.len(), 15);

        assert_eq!(r[0], Symbol("a".to_string()));
        assert_eq!(r[1], Int(12));
        assert_eq!(r[2], Symbol("d1-".to_string()));
        assert_eq!(r[3], Open);
        assert_eq!(r[4], Symbol("*".to_string()));
        assert_eq!(r[5], Complex(0., 1.));
        assert_eq!(r[6], Float(2.));
        assert_eq!(r[7], Open);
        assert_eq!(r[8], Symbol("+".to_string()));
        assert_eq!(r[9], Symbol("x".to_string()));
        assert_eq!(r[10], Int(3));
        assert_eq!(r[11], Close);
        assert_eq!(r[12], Close);
        assert_eq!(r[13], Open);
        assert_eq!(r[14], Close);
    }

}
