use crate::lex::{Token, TokenValue};
use crate::parse::ParseTree::Branch;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseTree {
    Leaf(Token),
    Branch(Vec<ParseTree>, u32, u32, Option<Box<ParseTree>>),
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
        TokenValue::Open => list(rest, tokens[0].line()),
        TokenValue::Close => Err((tokens[0].line(), "Unexpected end of list.".to_string())),
        _ => Ok((ParseTree::Leaf(tokens[0].clone()), rest)),
    }
}

fn list(tokens: &[Token], start_line: u32) -> Result<(ParseTree, &[Token]), (u32, String)> {
    let mut vals: Vec<ParseTree> = Vec::new();
    let mut t = &tokens[..];

    while !t.is_empty() {
        if t[0].value() == TokenValue::Close {
            return Ok((
                ParseTree::Branch(vals, start_line, t[0].line(), None),
                &t[1..],
            ));
        } else if t[0].value() == TokenValue::Cons {
            // Handle it.
            let (consed, rest) = statement(&t[1..])?;
            if rest.is_empty() {
                return Err((t[0].line(), String::from("Expected EOF.")));
            } else if rest[0].value() != TokenValue::Close {
                return Err((
                    t[0].line(),
                    String::from("Expected end of list following cons."),
                ));
            }
            return Ok((
                Branch(vals, start_line, rest[0].line(), Some(Box::new(consed))),
                &rest[1..],
            ));
        }

        let r = statement(t)?;

        t = r.1;
        vals.push(r.0);
    }

    Err((0, "Unexpected EOF at end of list.".to_string()))
}

#[cfg(test)]
mod test {
    use crate::lex::{start, Token, TokenValue::*};
    use crate::parse::ParseTree::{Branch, Leaf};
    use crate::parse::{parse, ParseTree};

    #[test]
    fn single_terminal() {
        let tokens = vec![Token::from(Int(16))];

        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], Leaf(Token::from(Int(16))));
    }

    #[test]
    fn empty_list() {
        let tokens = vec![Token::from(Open), Token::from(Close)];

        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 1);
        assert_eq!(x[0], Branch(Vec::new(), 0, 0, None));
    }

    #[test]
    fn list_of_terminals() {
        let tokens = vec![
            Token::from(Open),
            Token::from(Symbol("+".to_string())),
            Token::from(Int(16)),
            Token::from(Int(4)),
            Token::from(Close),
        ];

        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 1);

        match &x[0] {
            Branch(x, 0, 0, None) => {
                assert_eq!(x.len(), 3);
                assert_eq!(x[0], Leaf(Token::from(Symbol("+".to_string()))));
                assert_eq!(x[1], Leaf(Token::from(Int(16))));
                assert_eq!(x[2], Leaf(Token::from(Int(4))));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn nested_list() {
        let tokens = vec![
            Token::from(Open),
            Token::from(Open),
            Token::from(Close),
            Token::from(Open),
            Token::from(Open),
            Token::from(Close),
            Token::from(Close),
            Token::from(Close),
        ];

        let x = parse(&tokens).unwrap();

        assert_eq!(x.len(), 1);

        match &x[0] {
            Branch(x, 0, 0, None) => {
                assert_eq!(x.len(), 2);
                assert_eq!(x[0], Branch(Vec::new(), 0, 0, None));
                assert_eq!(
                    x[1],
                    Branch(vec!(Branch(Vec::new(), 0, 0, None)), 0, 0, None)
                );
            }
            _ => assert!(false),
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
            Branch(x, 1, 1, None) => {
                assert_eq!(x.len(), 4);
                assert_eq!(x[0], Leaf(tokens[4].clone()));
                assert_eq!(x[1], Leaf(tokens[5].clone()));
                assert_eq!(x[2], Leaf(tokens[6].clone()));

                match &x[3] {
                    Branch(x, 1, 1, None) => {
                        assert_eq!(x.len(), 3);

                        assert_eq!(x[0], Leaf(tokens[8].clone()));
                        assert_eq!(x[1], Leaf(tokens[9].clone()));
                        assert_eq!(x[2], Leaf(tokens[10].clone()));
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }

        match &x[4] {
            Branch(x, 1, 1, None) => {
                assert_eq!(x.len(), 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    #[should_panic]
    fn fails_unbalanced_parens() {
        parse(&vec![
            Token::from(Open),
            Token::from(Open),
            Token::from(Close),
        ])
        .unwrap();
    }

    #[test]
    fn cons_alone() {
        let tokens = start("(. zs)").unwrap();
        let x = parse(&tokens).unwrap();

        assert_eq!(1, x.len());
        assert_eq!(
            Branch(
                Vec::new(),
                1,
                1,
                Some(Box::new(Leaf(Token {
                    line: 1,
                    value: Symbol("zs".to_string())
                })))
            ),
            x[0]
        )
    }

    #[test]
    fn empty_lambda() {
        let tokens = start("(lambda () nil)").unwrap();
        let x = parse(&tokens).unwrap();
        assert_eq!(1, x.len());

        if let Branch(lambda_expr, 1, 1, None) = &x[0] {
            assert_eq!(3, lambda_expr.len());
            assert_eq!(
                &Leaf(Token {
                    line: 1,
                    value: Symbol("lambda".to_string())
                }),
                &lambda_expr[0]
            );
            assert_eq!(&Branch(Vec::new(), 1, 1, None), &lambda_expr[1]);
            assert_eq!(
                &Leaf(Token {
                    line: 1,
                    value: Nil
                }),
                &lambda_expr[2]
            )
        } else {
            panic!("Expected list")
        }
    }

    #[test]
    fn vararg_list() {
        let tokens = start("(a b c d . e)").unwrap();
        let x = parse(&tokens).unwrap();

        assert_eq!(1, x.len());

        if let Branch(args, 1, 1, Some(t)) = &x[0] {
            assert_eq!(4, args.len());
            match t.as_ref() {
                Leaf(t) if t.value() == Symbol("e".to_string()) => (),
                _ => panic!(),
            }
            match &args[0] {
                Leaf(t) if t.value() == Symbol("a".to_string()) => (),
                _ => panic!(),
            }
            match &args[1] {
                Leaf(t) if t.value() == Symbol("b".to_string()) => (),
                _ => panic!(),
            }
            match &args[2] {
                Leaf(t) if t.value() == Symbol("c".to_string()) => (),
                _ => panic!(),
            }
            match &args[3] {
                Leaf(t) if t.value() == Symbol("d".to_string()) => (),
                _ => panic!(),
            }
        }
    }

    #[test]
    #[should_panic]
    fn continued_vararg_list() {
        let tokens = start("(a b c . d e)").unwrap();
        let x = parse(&tokens).unwrap();
    }

    #[test]
    #[should_panic]
    fn non_continued_vararg_list() {
        let tokens = start("(a b c .)").unwrap();
        let x = parse(&tokens).unwrap();
    }
}
