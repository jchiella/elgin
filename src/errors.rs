//! Errors

use crate::lexer::{Span, Token};
use crate::parser::Node;

#[derive(Debug)]
pub enum Error {
    ExpectedToken { expected: Token, found: Span },
    EOF { lineno: usize, charno: usize },
    ExpectedIdent { found: Span },
    ExpectedType { found: Span },
    InvalidAtTopLevel { node: Node },
    VarNotInScope { name: String },
    TypeError,
    NoProc { name: String },
}
