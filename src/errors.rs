//! Errors

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
    },
    ExpectedIdent {
        found: Span,
    }
}

