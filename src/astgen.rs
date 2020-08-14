//! AST Generation

use crate::parser::Parser;
use crate::errors::{Logger, Span};
use crate::lexer::Token;
use crate::types::Type;

impl<'p> Parser<'p> {
    pub fn new(tokens: &'p [Span<Token>]) -> Self {
        Parser { 
            tokens, 
            index: 0,
        }
    }

    pub fn next(&mut self) -> Span<Token> {
        self.index += 1;
        if self.index >= self.tokens.len() {
            let last = self.tokens.last().unwrap();
            return Span {
                contents: Token::EOF,
                pos: last.pos,
                len: last.len,
            };
        }
        self.tokens[self.index - 1].clone() }
    pub fn peek(&mut self) -> Span<Token> {
        if self.index >= self.tokens.len() {
            let last = self.tokens.last().unwrap();
            return Span {
                contents: Token::EOF,
                pos: last.pos,
                len: last.len,
            };
        }
        self.tokens[self.index].clone()
    }

    pub fn ensure_next(&mut self, t: Token) -> Option<()> {
        if self.peek().contents == t {
            self.next();
            Some(())
        } else {
            Logger::syntax_error(
                format!("Expected a {:?} token, but found a {:?} instead", t, self.peek().contents.clone()).as_str(),
                self.peek().pos,
                self.peek().len,
            );
            None
        }
    }

    pub fn try_next(&mut self, t: Token) -> Option<()> {
        if self.peek().contents == t {
            self.next();
            Some(())
        } else {
            None
        }
    }

    pub fn ensure_ident(&mut self) -> Option<String> {
        if let Token::Ident(id) = self.peek().contents.clone() {
            self.next();
            Some(id)
        } else {
            Logger::syntax_error(
                format!("Expected an identifier, but found a {:?} token instead", self.peek().contents.clone()).as_str(),
                self.peek().pos,
                self.peek().len,
            );
            None
        }
    }

    pub fn ensure_type(&mut self) -> Option<Type> {
        match self.peek().contents.clone() {
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
                        Logger::syntax_error(
                            format!("Expected a type, but found a {:?} instead", self.peek().contents.clone()).as_str(),
                            self.peek().pos,
                            self.peek().len,
                        );
                        return None
                    }
                };
                self.next();
                Some(typ)
            },
            Token::Op(s) if s == "*" => {
                self.next();
                let content_type = self.ensure_type()?;
                Some(Type::Ptr(Box::new(content_type)))
            },
            Token::LBracket => {
                self.next(); // skip the LBracket
                if let Token::IntLiteral(size) = self.peek().contents {
                    self.next();
                    self.ensure_next(Token::RBracket)?;
                    let content_type = self.ensure_type()?; 
                    Some(Type::Array(size.parse().unwrap(), Box::new(content_type)))
                } else {
                    Logger::syntax_error(
                        format!("Expect an integer as the length of an array, but found a {:?} token instead", self.peek().contents).as_str(),
                        self.peek().pos,
                        self.peek().len,
                    );
                    None
                }
            },
            _ => {
                Logger::syntax_error(
                    format!("Expected a type, but found a {:?} instead", self.peek().contents.clone()).as_str(),
                    self.peek().pos,
                    self.peek().len,
                );
                None
            },
        }
    }
}
