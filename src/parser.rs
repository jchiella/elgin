//! The Elgin parser

use crate::errors::Error;
use crate::lexer::{Span, Token};

use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    IntLiteral,
    FloatLiteral,
    StrLiteral,

    I8,
    I16,
    I32,
    I64,
    I128,

    N8,
    N16,
    N32,
    N64,
    N128,

    F32,
    F64,
    F128,

    Bool,

    Variable(usize),

    Unknown,
    Undefined,
    NoReturn,

    Ptr(Box<Type>),
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Type::*;
        match self {
            IntLiteral => write!(f, "intLiteral"),
            FloatLiteral => write!(f, "floatLiteral"),
            StrLiteral => write!(f, "strLiteral"),

            I8 => write!(f, "i8"),
            I16 => write!(f, "i16"),
            I32 => write!(f, "i32"),
            I64 => write!(f, "i64"),
            I128 => write!(f, "i128"),

            N8 => write!(f, "n8"),
            N16 => write!(f, "n16"),
            N32 => write!(f, "n32"),
            N64 => write!(f, "n64"),
            N128 => write!(f, "n128"),

            F32 => write!(f, "f32"),
            F64 => write!(f, "f64"),
            F128 => write!(f, "f128"),

            Bool => write!(f, "bool"),

            Ptr(t) => write!(f, "*{:?}", t),

            Variable(n) => write!(f, "${}", n),

            Unknown => write!(f, "UNKNOWN"),
            Undefined => write!(f, "undefined"),
            NoReturn => write!(f, "noreturn"),
        }?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
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
    },
    IfStatement {
        condition: Box<Node>,
        body: Box<Node>,
        else_body: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    WhileStatement {
        condition: Box<Node>,
        body: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    Block {
        nodes: Vec<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    VarStatement {
        name: String,
        typ: Type,
        value: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    ConstStatement {
        name: String,
        typ: Type,
        value: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    AssignStatement {
        name: String,
        value: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    ProcStatement {
        name: String,
        args: Vec<String>,
        arg_types: Vec<Type>,
        ret_type: Type,
        body: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
    ReturnStatement {
        val: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    },
}

pub struct Parser<'p> {
    tokens: &'p [Span],
    index: usize,
}

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p [Span]) -> Self {
        Parser { tokens, index: 0 }
    }

    fn next(&mut self) -> Span {
        self.index += 1;
        if self.index >= self.tokens.len() {
            let last = self.tokens.last().unwrap();
            return Span {
                token: Token::EOF,
                lineno: last.lineno,
                start: last.start,
                end: last.end,
            };
        }
        self.tokens[self.index - 1].clone()
    }

    fn peek(&mut self) -> Span {
        if self.index >= self.tokens.len() {
            let last = self.tokens.last().unwrap();
            return Span {
                token: Token::EOF,
                lineno: last.lineno,
                start: last.start,
                end: last.end,
            };
        }
        self.tokens[self.index].clone()
    }

    fn ensure_next(&mut self, t: Token) -> Result<(), Error> {
        if self.peek().token == t {
            self.next();
            Ok(())
        } else {
            Err(Error::ExpectedToken {
                expected: t,
                found: self.peek().clone(),
            })
        }
    }

    fn ensure_ident(&mut self) -> Result<String, Error> {
        if let Token::Ident(id) = self.peek().token.clone() {
            self.next();
            Ok(id)
        } else {
            Err(Error::ExpectedIdent {
                found: self.peek().clone(),
            })
        }
    }

    fn ensure_type(&mut self) -> Result<Type, Error> {
        match self.peek().token.clone() {
            Token::Ident(id) => {
                let typ = match id.as_str() {
                    "i8" => Type::I8,
                    "i16" => Type::I16,
                    "i32" => Type::I32,
                    "i64" => Type::I64,
                    "i128" => Type::I128,

                    "n8" => Type::N8,
                    "n16" => Type::N16,
                    "n32" => Type::N32,
                    "n64" => Type::N64,
                    "n128" => Type::N128,

                    "f32" => Type::F32,
                    "f64" => Type::F64,
                    "f128" => Type::F128,

                    "bool" => Type::Bool,

                    _ => {
                        return Err(Error::ExpectedType {
                            found: self.peek().clone(),
                        })
                    }
                };
                self.next();
                Ok(typ)
            },
            Token::Op(s) if s == "*" => {
                self.next();
                let content_type = self.ensure_type()?;
                Ok(Type::Ptr(Box::new(content_type)))
            },
            _ => {
                Err(Error::ExpectedType {
                    found: self.peek().clone(),
                })
            },
        }
    }

    pub fn go(&mut self) -> Result<Vec<Node>, Error> {
        let mut nodes = vec![];
        loop {
            match self.peek().token {
                Token::DocComment(_) => {
                    self.next(); // one day there will be doc comment support
                },
                Token::Newline => {
                    self.next();
                },
                _ => {
                    nodes.push(self.statement()?);
                    self.ensure_next(Token::Newline)?;
                }
            };
            if self.peek().token == Token::EOF {
                break;
            }
        }
        Ok(nodes)
    }

    fn statement(&mut self) -> Result<Node, Error> {
        Ok(match self.peek().token {
            Token::If => self.if_statement(true)?,
            Token::While => self.while_statement()?,
            Token::Loop => self.loop_statement()?,
            Token::Var => self.var_statement()?,
            Token::Const => self.const_statement()?,
            Token::Proc => self.proc_statement()?,
            Token::Return => self.return_statement()?,
            Token::Ident(_) if self.tokens[self.index + 1].token == Token::Equals => {
                self.assign_statement()?
            }
            _ => self.expr(0)?,
        })
    }

    fn if_statement(&mut self, ensure_if: bool) -> Result<Node, Error> {
        if ensure_if {
            self.ensure_next(Token::If)?;
        }
        let condition = self.expr(0)?;
        let body = self.block()?;
        let else_body;
        if self.peek().token == Token::Elif {
            self.ensure_next(Token::Elif)?;
            else_body = self.if_statement(false)?;
        } else if self.peek().token == Token::Else {
            self.ensure_next(Token::Else)?;
            else_body = self.block()?;
        } else {
            else_body = Node::Block {
                nodes: vec![Node::Literal {
                    typ: Type::Undefined,
                    value: "undefined".to_owned(),
                    lineno: 0,
                    start: 0,
                    end: 0,
                }],
                lineno: 0,
                start: 0,
                end: 0,
            };
        }

        Ok(Node::IfStatement {
            condition: Box::new(condition),
            body: Box::new(body.clone()),
            else_body: Box::new(else_body),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn while_statement(&mut self) -> Result<Node, Error> {
        self.ensure_next(Token::While)?;
        let condition = self.expr(0)?;
        let body = self.block()?;

        Ok(Node::WhileStatement {
            condition: Box::new(condition),
            body: Box::new(body.clone()),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn loop_statement(&mut self) -> Result<Node, Error> {
        self.ensure_next(Token::Loop)?;
        let condition = Node::Literal {
            typ: Type::Bool,
            value: "true".to_owned(),
            lineno: 0,
            start: 0,
            end: 0,
        };
        let body = self.block()?;

        Ok(Node::WhileStatement {
            condition: Box::new(condition),
            body: Box::new(body.clone()),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn block(&mut self) -> Result<Node, Error> {
        let mut nodes = vec![];
        self.ensure_next(Token::LBrace)?;
        loop {
            let _ = self.ensure_next(Token::Newline);
            nodes.push(self.statement()?);
            if self.ensure_next(Token::Newline).is_err() {
                self.ensure_next(Token::RBrace)?;
                break;
            }
            if self.peek().token == Token::RBrace {
                self.ensure_next(Token::RBrace)?;
                break;
            }
        }
        Ok(Node::Block {
            nodes,
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn var_statement(&mut self) -> Result<Node, Error> {
        self.ensure_next(Token::Var)?;
        let name = self.ensure_ident()?;
        let typ;
        if self.ensure_next(Token::Colon).is_ok() {
            typ = self.ensure_type()?;
        } else {
            typ = Type::Unknown;
        }
        let value;
        if self.peek().token == Token::Equals {
            self.ensure_next(Token::Equals)?;
            value = self.expr(0)?;
        } else {
            value = Node::Literal {
                typ: Type::Undefined,
                value: "undefined".to_owned(),
                lineno: 0,
                start: 0,
                end: 0,
            };
        }

        Ok(Node::VarStatement {
            name,
            typ,
            value: Box::new(value),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn assign_statement(&mut self) -> Result<Node, Error> {
        let name = self.ensure_ident()?;
        self.ensure_next(Token::Equals)?;
        let value = self.expr(0)?;

        Ok(Node::AssignStatement {
            name,
            value: Box::new(value),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn const_statement(&mut self) -> Result<Node, Error> {
        self.ensure_next(Token::Const)?;
        let name = self.ensure_ident()?;
        let typ;
        if self.ensure_next(Token::Colon).is_ok() {
            typ = self.ensure_type()?;
        } else {
            typ = Type::Unknown;
        }
        self.ensure_next(Token::Equals)?;
        let value = self.expr(0)?;

        Ok(Node::ConstStatement {
            name,
            typ,
            value: Box::new(value),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn proc_statement(&mut self) -> Result<Node, Error> {
        self.ensure_next(Token::Proc)?;
        let name = self.ensure_ident()?;
        self.ensure_next(Token::LParen)?;
        let mut args = vec![];
        let mut arg_types = vec![];
        while self.peek().token != Token::RParen {
            args.push(self.ensure_ident()?);
            self.ensure_next(Token::Colon)?;
            arg_types.push(self.ensure_type()?);
            if self.peek().token != Token::Comma {
                break;
            } else {
                self.ensure_next(Token::Comma)?;
            }
        }
        self.ensure_next(Token::RParen)?;
        let ret_type;
        if self.ensure_next(Token::Colon).is_ok() {
            ret_type = self.ensure_type()?;
        } else {
            ret_type = Type::Undefined;
        }
        let body;
        if self.peek().token == Token::LBrace {
            body = self.block()?;
        } else {
            body = Node::Block {
                nodes: vec![],
                lineno: 0, start: 0, end: 0,
            }
        }

        Ok(Node::ProcStatement {
            name,
            args,
            arg_types,
            ret_type,
            body: Box::new(body),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn return_statement(&mut self) -> Result<Node, Error> {
        self.ensure_next(Token::Return)?;
        let val = self.expr(0)?;
        Ok(Node::ReturnStatement {
            val: Box::new(val),
            lineno: 0,
            start: 0,
            end: 0,
        })
    }

    fn expr(&mut self, min_bp: u8) -> Result<Node, Error> {
        let mut left = match self.next().clone() {
            Span {
                token: Token::Ident(id),
                lineno,
                start,
                end,
            } => {
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
                        args,
                        lineno,
                        start,
                        end,
                    }
                } else {
                    Node::VariableRef {
                        name: id,
                        lineno,
                        start,
                        end,
                    }
                }
            }
            Span {
                token: Token::IntLiteral(int),
                lineno,
                start,
                end,
            } => Node::Literal {
                typ: Type::IntLiteral,
                value: int,
                lineno,
                start,
                end,
            },
            Span {
                token: Token::FloatLiteral(float),
                lineno,
                start,
                end,
            } => Node::Literal {
                typ: Type::FloatLiteral,
                value: float,
                lineno,
                start,
                end,
            },
            Span {
                token: Token::StrLiteral(s),
                lineno,
                start,
                end,
            } => Node::Literal {
                typ: Type::StrLiteral,
                value: s,
                lineno,
                start,
                end,
            },
            Span {
                token: Token::LParen,
                ..
            } => {
                let left = self.expr(0)?;
                self.ensure_next(Token::RParen)?;
                left
            }
            Span {
                token: Token::Op(op),
                lineno,
                start,
                end,
            } => {
                let ((), right_bp) = prefix_binding_power(&op);
                let right = self.expr(right_bp)?;
                Node::PrefixOp {
                    op,
                    right: Box::new(right),
                    lineno,
                    start,
                    end,
                }
            }
            Span {
                token: Token::EOF,
                lineno,
                end,
                ..
            } => {
                return Err(Error::EOF {
                    lineno,
                    charno: end,
                })
            }
            t => panic!("Bad token: {:?}", t),
        };

        loop {
            let op = match self.peek().token.clone() {
                Token::EOF
                | Token::Newline
                | Token::RParen
                | Token::RBracket
                | Token::Comma
                | Token::LBrace
                | Token::RBrace => break,
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
                    Node::IndexOp {
                        object: Box::new(left),
                        index: Box::new(right),
                        lineno: 0,
                        start: 0,
                        end: 0,
                    }
                } else {
                    Node::PostfixOp {
                        op,
                        left: Box::new(left),
                        lineno: 0,
                        start: 0,
                        end: 0,
                    }
                };
                continue;
            }

            if let Some((left_bp, right_bp)) = infix_binding_power(&op) {
                if left_bp < min_bp {
                    break;
                }
                self.next();

                let right = self.expr(right_bp)?;
                left = Node::InfixOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    lineno: 0,
                    start: 0,
                    end: 0,
                };
                continue;
            }

            break;
        }

        Ok(left)
    }
}

fn prefix_binding_power(op: &String) -> ((), u8) {
    match op.as_str() {
        "!" => ((), 8),
        "+" | "-" => ((), 9),
        o => unreachable!(o),
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
        ">" | "<" | ">=" | "<=" | "==" | "!=" => (3, 4),
        "+" | "-" => (5, 6),
        "*" | "/" => (7, 8),
        _ => return None,
    })
}
