//! The Chi parser

use crate::lexer::{Token, Span};

#[derive(Debug)]
pub enum Error {
    ExpectedToken {
        expected: Token,
        found: Span,
    },
    EOF {
        lineno: usize,
        charno: usize,
    }
}

#[derive(Debug)]
pub enum Type {
    Int,
    Float,
    Str,
}

#[derive(Debug)]
pub enum Node {
    Literal {
        typ: Type,
        value: String,
        lineno: usize,
        start: usize,
        end: usize,
    },
    Call {
        name: String,
        args: Vec<Node>, 
        lineno: usize, 
        start: usize, 
        end: usize,
    },
    InfixOp {
        op: String,
        left: Box<Node>,
        right: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    PrefixOp {
        op: String,
        right: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    PostfixOp {
        op: String,
        left: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    IndexOp {
        object: Box<Node>,
        index: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    VariableRef {
        name: String,
        lineno: usize,
        start: usize,
        end: usize,
    }
}

pub struct Parser<'p> {
    tokens: &'p [Span],
    index: usize,
}

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p [Span]) -> Self {
        Parser {
            tokens,
            index: 0,
        }
    }

    fn next(&mut self) -> &Span {
        self.index += 1;
        if self.index >= self.tokens.len() {
            return &Span {token: Token::EOF, lineno: 0, start: 0, end: 0}
        }
        &self.tokens[self.index - 1]
    }

    fn peek(&mut self) -> &Span {
        if self.index >= self.tokens.len() {
            return &Span {token: Token::EOF, lineno: 0, start: 0, end: 0}
        }
        &self.tokens[self.index]
    }

    fn ensure_next(&mut self, t: Token) -> Result<(), Error> {
        if self.peek().token == t {
            self.next();
            Ok(())
        } else {
            Err(Error::ExpectedToken {expected: t, found: self.peek().clone()})
        }
    }

    pub fn go(&mut self) -> Result<Vec<Node>, Error> {
        let mut nodes = vec![];
        loop {
            nodes.push(self.expr(0)?);
            self.ensure_next(Token::Newline)?;
            if self.peek().token == Token::EOF {
                break;
            }
        }
        Ok(nodes)
    }

    fn expr(&mut self, min_bp: u8) -> Result<Node, Error> {
        let mut left = match self.next().clone() {
            Span{ token: Token::Ident(id), lineno, start, end } => {
                if self.peek().token == Token::LParen {
                    self.next(); // pass the LParen;
                    let mut args = Vec::new();
                    while self.peek().token != Token::RParen {
                        args.push(self.expr(0)?);
                        if self.peek().token != Token::Comma {
                            break;
                        } else {
                            self.ensure_next(Token::Comma)?;
                        }
                    }
                    self.ensure_next(Token::RParen)?;
                    Node::Call {
                        name: id,
                        args, lineno, start, end
                    }
                } else {
                    Node::VariableRef {
                          name: id,
                          lineno, start, end 
                    }
                }
            },
            Span{ token: Token::IntLiteral(int), lineno, start, end } => Node::Literal {
                      typ: Type::Int,
                      value: int,
                      lineno, start, end },
            Span{ token: Token::FloatLiteral(float), lineno, start, end } => Node::Literal {
                      typ: Type::Float,
                      value: float,
                      lineno, start, end },
            Span{ token: Token::StrLiteral(s), lineno, start, end } => Node::Literal {
                      typ: Type::Str,
                      value: s,
                      lineno, start, end },
            Span { token: Token::LParen, .. } => {
                let left = self.expr(0)?;
                self.ensure_next(Token::RParen)?;
                left
            },
            Span { token: Token::Op(op), lineno, start, end } => {
                let ((), right_bp) = prefix_binding_power(&op);
                let right = self.expr(right_bp)?;
                Node::PrefixOp { op, right: Box::new(right), lineno, start, end }
            },
            Span { token: Token::EOF, lineno, start, .. } => return Err(Error::EOF { lineno, charno: start }),
            t => panic!("Bad token: {:?}", t),
        };

        loop {
            let op = match self.peek().token.clone() {
                Token::EOF | Token::Newline | Token::RParen | Token::RBracket | Token::Comma => break,
                Token::Op(op) => op,
                Token::LBracket => "[".to_owned(),
                t => panic!("Bad token: {:?}", t),
            };
            
            if let Some((left_bp, ())) = postfix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }
                self.next();

                left = if op == "[" {
                    let right = self.expr(0)?;
                    self.ensure_next(Token::RBracket)?;
                    Node::IndexOp { object: Box::new(left), index: Box::new(right), lineno: 0, start: 0, end: 0 }
                } else {
                    Node::PostfixOp { op, left: Box::new(left), lineno: 0, start: 0, end: 0 }
                };
                continue;
            }

            if let Some((left_bp, right_bp)) = infix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }
                self.next();

                let right = self.expr(right_bp)?;
                left = Node::InfixOp { op, left: Box::new(left), right: Box::new(right), lineno: 0, start: 0, end: 0 };
                continue;
            }

            break;
        }

        Ok(left)
    }
}

fn prefix_binding_power(op: &String) -> ((), u8) {
    match op.as_str() {
        "+" | "-" => ((), 9),
        _ => unreachable!(),
    }
}

fn postfix_binding_power(op: &String) -> Option<(u8, ())> {
    Some(match op.as_str() {
        "[" => (11, ()),
        _ => return None,
    })
}

fn infix_binding_power(op: &String) -> Option<(u8, u8)> {
    Some(match op.as_str() {
        "+" | "-" => (5, 6),
        "*" | "/" => (7, 8),
        _ => return None,
    })
}
