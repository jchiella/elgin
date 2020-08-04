//! The static analysis component of Chi
//! Does fun tests like type checking and more...

use crate::errors::Error;
use crate::parser::Node;

pub struct Analyzer<'a> {
    ast: &'a [Node],
}

impl<'a> Analyzer<'a> {
    pub fn new(ast: &'a [Node]) -> Self {
        Analyzer {
            ast,
        }
    }

    pub fn go(&self) -> Result<(), Error> {
        Ok(())
    }
}
