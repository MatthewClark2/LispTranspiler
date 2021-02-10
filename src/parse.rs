use crate::lex::{Token, TokenValue};

#[derive(Debug, PartialEq, Clone)]
pub enum ParseTree {
    Leaf(Token),
    Branch(Vec<ParseTree>),
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<ParseTree>, (u32, String)> {
    let mut statements: Vec<ParseTree> = Vec::new();
    let mut t = &tokens[..];

    while !t.is_empty() {
        let r = statement(t)?;
        t = r.1;
        statements.push(r.0);
    }

    Ok(statements)
}

// The following auxiliary functions expect at least one token to be present.
fn statement(tokens: &[Token]) -> Result<(ParseTree, &[Token]), (u32, String)> {
    let rest = &tokens[1..];

    match tokens[0].value() {
        TokenValue::Open => list(rest),
        TokenValue::Close => Err((tokens[0].line(), "Unexpected end of list.".to_string())),
        _ => Ok((ParseTree::Leaf(tokens[0].clone()), rest))
    }
}

fn list(tokens: &[Token]) -> Result<(ParseTree, &[Token]), (u32, String)> {
    let mut vals: Vec<ParseTree> = Vec::new();
    let mut t = &tokens[..];

    while !t.is_empty() {
        if t[0].value() == TokenValue::Close {
            return Ok((ParseTree::Branch(vals), &t[1..]))
        }

        let r = statement(t)?;
        t = r.1;
        vals.push(r.0);
    }

    Err((0, "Unexpected EOF at end of list.".to_string()))
}

#[cfg(test)]
mod test {
    use crate::parse::parse;
    use crate::lex::{Token, TokenValue};
    use crate::parse::ParseTree::{Leaf, Branch};

    #[test]
    fn single_terminal() {
        let tokens = vec!(Token::from(TokenValue::Int(16)));

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], Leaf(Token::from(TokenValue::Int(16))));
    }

    #[test]
    fn empty_list() {
        let tokens = vec!(Token::from(TokenValue::Open), Token::from(TokenValue::Close));

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], Branch(Vec::new()));
    }

    #[test]
    fn list_of_terminals() {
        let tokens = vec!(
            Token::from(TokenValue::Open),
            Token::from(TokenValue::Symbol("+".to_string())),
            Token::from(TokenValue::Int(16)),
            Token::from(TokenValue::Int(4)),
            Token::from(TokenValue::Close));

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);

        match &x[0] {
            Leaf(_) => assert!(false),
            Branch(x) => {
                assert_eq!(x.len(), 3);
                assert_eq!(x[0], Leaf(Token::from(TokenValue::Symbol("+".to_string()))));
                assert_eq!(x[1], Leaf(Token::from(TokenValue::Int(16))));
                assert_eq!(x[2], Leaf(Token::from(TokenValue::Int(4))));
            }
        }
    }

    #[test]
    fn nested_list() {
        let tokens = vec!(
            Token::from(TokenValue::Open),
            Token::from(TokenValue::Open),
            Token::from(TokenValue::Close),
            Token::from(TokenValue::Open),
            Token::from(TokenValue::Open),
            Token::from(TokenValue::Close),
            Token::from(TokenValue::Close),
            Token::from(TokenValue::Close),
        );

        let x = parse(tokens).unwrap();

        assert_eq!(x.len(), 1);

        match &x[0] {
            Leaf(_) => assert!(false),
            Branch(x) => {
                assert_eq!(x.len(), 2);
                assert_eq!(x[0], Branch(Vec::new()));
                assert_eq!(x[1], Branch(vec!(Branch(Vec::new()))));
            }
        }
    }

    /*
    #[test]
    #[ignore]
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
    */

    #[test]
    #[should_panic]
    fn fails_unbalanced_parens() {
        parse(vec!(
            Token::from(TokenValue::Open),
            Token::from(TokenValue::Open),
            Token::from(TokenValue::Close),
        )).unwrap();
    }
}
