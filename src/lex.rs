// TODO(matthew-c21): Consider expanding with longs and arbitrary precision types.
pub enum Number {
    Int(i32),
    Float(f64),
    Complex(i32, i32),
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
            x if is_space(&x) => continue,
            x if is_numeric(&x) => consume_number(i),
            x if is_alpha(&x) => consume_symbol(i),
            _ => Err(format!("Unrecognized character `{}`", c)),
        }?;

        tokens.push(r.0);
        i = r.1;
    }

    Ok(tokens)
}

// TODO(matthew-c21): Rust probably has these built in better.
fn is_numeric(s: &char) -> bool {
    ('0'..'9').contains(s)
}

fn is_alpha(s: &char) -> bool {
    ('a'..'z').contains(s) || ('A'..'Z').contains(s)
}

fn is_space(s: &char) -> bool {
    [' ', '\t', '\n', '\r'].contains(s)
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
    Err(input.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn lparen_only() {}

    #[test]
    fn rparen_only() {}

    #[test]
    fn valid_escaped_string() {}

    #[test]
    fn invalid_escaped_string() {}

    #[test]
    fn valid_numbers() {}

    #[test]
    fn invalid_numbers() {}

    #[test]
    fn valid_symbols() {}

    #[test]
    fn invalid_symbols() {}

    #[test]
    fn valid_keywords() {}

    #[test]
    fn invalid_keywords() {}
}
