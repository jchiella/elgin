//! The Chi lexer

use std::fmt;

use crate::errors::Error;

const SPECIAL_CHARS: [char; 9] = ['(', ')', '[', ']', '{', '}', ',', '=', ':'];

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // literals
    IntLiteral(String),
    FloatLiteral(String),
    StrLiteral(String),

    // identifier
    Ident(String),

    // operator
    Op(String),
    
    // keywords
    Proc,
    If,
    Elif,
    Else,
    While,
    Loop,
    Var,
    Const,

    // special characters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Equals,
    Colon,

    // newline
    Newline,

    // end of file (used by parser)
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct Span {
    pub token: Token,
    pub lineno: usize,
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} @ line {}, from {}-{}", self.token, self.lineno, self.start, self.end)
    }
}

pub struct Lexer<'l> {
    code: &'l [char],
    index: usize,
    lineno: usize,
    charno: usize,
    nesting: usize,
}

impl<'l> Lexer<'l> {
    pub fn new(code: &'l [char]) -> Self {
        Lexer {
            code,
            index: 0,
            lineno: 0,
            charno: 0,
            nesting: 0,
        }
    } 

    fn peek(&self) -> char {
        if self.index >= self.code.len() {
            return '\0';
        }
        self.code[self.index]
    }

    fn next(&mut self) -> char {
        self.index += 1;
        if self.index >= self.code.len() {
            return '\0';
        }
        let ch = self.code[self.index - 1];
        match ch {
            '\n' => {
                self.lineno += 1;
                self.charno = 0;
            },
            _ => {
                self.charno += 1;
            }
        }
        ch
    }

    fn ident_str(&mut self) -> String {
        let mut ident = String::new();
        while is_ident(self.peek()) {
            ident.push(self.next());
        }
        ident
    } 

    fn number(&mut self) -> Token {
        let mut number = String::new();
        let mut decimal_passed = false;

        while is_number(self.peek(), decimal_passed) {
            number.push(match self.next() {
                '.' => {
                    decimal_passed = true;
                    '.'
                },
                c => c,
            });
        }
        if decimal_passed {
            Token::FloatLiteral(number)
        } else {
            Token::IntLiteral(number)
        }
    }
    
    fn operator(&mut self) -> Token {
        let mut op = String::new();
        while is_op(self.peek()) {
            op.push(self.next());
        }
        Token::Op(op)
    }

    fn string(&mut self) -> Result<Token, Error> {
        let mut string = String::new();
        self.next(); // skip "
        while self.peek() != '"' {
            if self.peek() == '\0' {
                return Err(Error::EOF {lineno: self.lineno, charno: self.charno});
            }
            string.push(self.next());
        }
        self.next(); // skip "
        Ok(Token::StrLiteral(string))

    }

    fn special(&mut self) -> Token {
        match self.peek() {
            '(' | '[' => self.nesting += 1,
            ')' | ']' => self.nesting -= 1,
            ',' | '=' | ':' | '{' | '}' => (),
            _ => unreachable!(),
        };
        match self.next() {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            '=' => Token::Equals,
            ':' => Token::Colon,
            _ => unreachable!(),
        }
    }

    pub fn go(&mut self) -> Result<Vec<Span>, Error> {
        let mut tokens = vec![];
        loop {
            match self.peek() {
                ch if is_ident_start(ch) => {
                    let id = self.ident_str();
                    tokens.push(self.spanned(str_to_keyword(&id)
                        .unwrap_or_else(|| str_to_ident(&id))));
                },
                ch if is_number(ch, false) => {
                    let number = self.number();
                    tokens.push(self.spanned(number));
                },
                ch if is_special(ch) => {
                    let special = self.special();
                    tokens.push(self.spanned(special));
                }
                '"' => {
                    let string = self.string()?;
                    tokens.push(self.spanned(string));
                },
                ch if is_op(ch) => {
                    let operator = self.operator();
                    tokens.push(self.spanned(operator));
                },
                ch if ch == '\n' => {
                    // token::proc doesn't matter, just needs to be
                    // something that doesn't trigger newline suppression
                    if tokens.last().unwrap().token == Token::Newline {
                        self.next(); // skip consecutive newlines
                    } else {
                        match tokens.last().unwrap_or(&Span {token: Token::Proc, lineno: 0, start: 0, end: 0}).token {
                            Token::Op(_) | Token::Comma => self.next(),
                            _ if self.nesting != 0 => self.next(),
                            _ => {
                                tokens.push(self.spanned(Token::Newline));
                                self.next()
                            },
                        };
                    }
                },
                ch if ch.is_ascii_whitespace() => {
                    self.next();
                },
                '\0' => break,
                _ => unreachable!(),
            }
        }
        Ok(tokens)
    }

    fn spanned(&mut self, token: Token) -> Span {
        Span {
            token: token.clone(),
            lineno: self.lineno + 1,
            start: 0,//self.charno - token_len(&token) + 1, 
            end: self.charno + 1,
        }
    }
}

#[inline]
fn is_ident(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[inline]
fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

#[inline]
fn is_number(ch: char, decimal_passed: bool) -> bool {
    ch.is_ascii_digit() || (ch == '.' && !decimal_passed)
}

#[inline]
fn is_special(ch: char) -> bool {
    SPECIAL_CHARS.contains(&ch)
}

#[inline]
fn is_op(ch: char) -> bool {
    ch.is_ascii_punctuation()
}

fn str_to_keyword(s: &str) -> Option<Token> {
    Some(match s {
        "proc" => Token::Proc, 
        "if" => Token::If, 
        "else" => Token::Else,
        "elif" => Token::Elif,
        "while" => Token::While,
        "loop" => Token::Loop,
        "var" => Token::Var,
        "const" => Token::Const,
        _ => return None,
    })
}

#[inline]
fn str_to_ident(s: &str) -> Token {
    Token::Ident(s.to_owned())
}

fn token_len(t: &Token) -> usize {
    match t {
        Token::IntLiteral(s) => s.len(),
        Token::FloatLiteral(s) => s.len(),
        Token::StrLiteral(s) => s.len(),

        Token::Ident(s) => s.len(),
        Token::Op(s) => s.len(),

        Token::Proc => 4,
        Token::If => 2,
        Token::Else => 4,
        Token::Elif => 4,
        Token::While => 5,
        Token::Loop => 4,
        Token::Var => 3,
        Token::Const => 5,

        Token::LParen 
            | Token::RParen 
            | Token::LBracket 
            | Token::RBracket 
            | Token::LBrace 
            | Token::RBrace 
            | Token::Comma 
            | Token::Equals 
            | Token::Colon => 1,

        // newline
        Token::Newline => 1,
        Token::EOF => unreachable!(),
    }
}
