use crate::data::LispDatum;
use crate::lex::Token;
use crate::parse::Statement::{Terminal, List};
use crate::data::LispDatum::{Symbol, Integer, Real, Complex, Rational};

#[derive(Debug, PartialEq, Clone)]
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
            list(rest)
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
        if t[0] == Token::Close {
            return Ok((List(vals), &t[1..]))
        }

        let r = statement(t)?;
        t = r.1;
        vals.push(r.0);
    }

    Err("Expected end of list".to_string())
}

#[cfg(test)]
mod test {
    use crate::parse::{parse, Statement};
    use crate::lex::Token;
    use crate::parse::Statement::{Terminal, List};
    use crate::data::LispDatum;
    use crate::data::LispDatum::{Symbol, Integer, Complex, Real};
    use crate::lex;

    #[test]
    fn single_terminal() {
        let tokens = vec!(Token::Int(16));

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], Terminal(LispDatum::Integer(16)));
    }

    #[test]
    fn empty_list() {
        let tokens = vec!(Token::Open, Token::Close);

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], List(Vec::new()));
    }

    #[test]
    fn list_of_terminals() {
        let tokens = vec!(Token::Open, Token::Symbol("+".to_string()), Token::Int(16), Token::Int(4), Token::Close);

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);

        match &x[0] {
            Terminal(_) => assert!(false),
            List(x) => {
                assert_eq!(x.len(), 3);
                assert_eq!(x[0], Terminal(Symbol("+".to_string())));
                assert_eq!(x[1], Terminal(Integer(16)));
                assert_eq!(x[2], Terminal(Integer(4)));
            }
        }
    }

    #[test]
    fn nested_list() {
        let tokens = vec!(Token::Open, Token::Open, Token::Close, Token::Open, Token::Open, Token::Close, Token::Close, Token::Close);

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);

        match &x[0] {
            Terminal(_) => assert!(false),
            List(x) => {
                assert_eq!(x.len(), 2);
                assert_eq!(x[0], List(Vec::new()));
                assert_eq!(x[1], List(vec!(List(Vec::new()))));
            }
        }
    }

    #[test]
    fn small_comprehensive() {
        let tokens = lex::start("a 12 d1- (* 1i 2. (+ x 3)) ()").unwrap();
        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 5);
        assert_eq!(x[0], Terminal(Symbol("a".to_string())));
        assert_eq!(x[1], Terminal(Integer(12)));
        assert_eq!(x[2], Terminal(Symbol("d1-".to_string())));
        match &x[3] {
            Terminal(_) => panic!(),
            List(x) => {
                assert_eq!(x.len(), 4);
                assert_eq!(x[0], Terminal(Symbol("*".to_string())));
                assert_eq!(x[1], Terminal(Complex(0., 1.)));
                assert_eq!(x[2], Terminal(Real(2.)));

                match &x[3] {
                    Terminal(_) => panic!(),
                    List(x) => {
                        assert_eq!(x.len(), 3);

                        assert_eq!(x[0], Terminal(Symbol("+".to_string())));
                        assert_eq!(x[1], Terminal(Symbol("x".to_string())));
                        assert_eq!(x[2], Terminal(Integer(3)));
                    }
                }
            }
        }

        match &x[4] {
            Terminal(_) => panic!(),
            List(x) => {
                assert_eq!(x.len(), 0);
            }
        }
    }

    #[test]
    #[should_panic]
    fn fails_unbalanced_parens() {
        parse(vec!(Token::Open, Token::Open, Token::Close)).unwrap();
    }
}
