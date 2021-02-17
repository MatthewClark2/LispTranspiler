use crate::lex::{Token, TokenValue};

#[derive(Debug, PartialEq, Clone)]
pub enum ParseTree {
    Leaf(Token),
    Branch(Vec<ParseTree>),
}

pub fn parse(tokens: &Vec<Token>) -> Result<Vec<ParseTree>, (u32, String)> {
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
    use crate::lex::{Token, TokenValue::*, start};
    use crate::parse::ParseTree::{Leaf, Branch};

    #[test]
    fn single_terminal() {
        let tokens = vec!(Token::from(Int(16)));

        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], Leaf(Token::from(Int(16))));
    }

    #[test]
    fn empty_list() {
        let tokens = vec!(Token::from(Open), Token::from(Close));

        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], Branch(Vec::new()));
    }

    #[test]
    fn list_of_terminals() {
        let tokens = vec!(
            Token::from(Open),
            Token::from(Symbol("+".to_string())),
            Token::from(Int(16)),
            Token::from(Int(4)),
            Token::from(Close));

        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 1);

        match &x[0] {
            Leaf(_) => assert!(false),
            Branch(x) => {
                assert_eq!(x.len(), 3);
                assert_eq!(x[0], Leaf(Token::from(Symbol("+".to_string()))));
                assert_eq!(x[1], Leaf(Token::from(Int(16))));
                assert_eq!(x[2], Leaf(Token::from(Int(4))));
            }
        }
    }

    #[test]
    fn nested_list() {
        let tokens = vec!(
            Token::from(Open),
            Token::from(Open),
            Token::from(Close),
            Token::from(Open),
            Token::from(Open),
            Token::from(Close),
            Token::from(Close),
            Token::from(Close),
        );

        let x = parse(&tokens).unwrap();

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

    #[test]
    fn small_comprehensive() {
        let tokens = start("a 12 d1- (* 1i 2. (+ x 3)) ()").unwrap();
        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 5);
        assert_eq!(x[0], Leaf(tokens[0].clone()));
        assert_eq!(x[1], Leaf(tokens[1].clone()));
        assert_eq!(x[2], Leaf(tokens[2].clone()));
        match &x[3] {
            Leaf(_) => panic!(),
            Branch(x) => {
                assert_eq!(x.len(), 4);
                assert_eq!(x[0], Leaf(tokens[4].clone()));
                assert_eq!(x[1], Leaf(tokens[5].clone()));
                assert_eq!(x[2], Leaf(tokens[6].clone()));

                match &x[3] {
                    Leaf(_) => panic!(),
                    Branch(x) => {
                        assert_eq!(x.len(), 3);

                        assert_eq!(x[0], Leaf(tokens[8].clone()));
                        assert_eq!(x[1], Leaf(tokens[9].clone()));
                        assert_eq!(x[2], Leaf(tokens[10].clone()));
                    }
                }
            }
        }

        match &x[4] {
            Leaf(_) => panic!(),
            Branch(x) => {
                assert_eq!(x.len(), 0);
            }
        }
    }

    #[test]
    #[should_panic]
    fn fails_unbalanced_parens() {
        parse(&vec!(
            Token::from(Open),
            Token::from(Open),
            Token::from(Close),
        )).unwrap();
    }
}
