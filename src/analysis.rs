//! The static analysis component of Chi
//! Does fun tests like type checking and more...

use crate::errors::Error;
use crate::parser::Node;

pub struct Analyzer<'a> {
    ast: &'a mut [Node],
}

impl<'a> Analyzer<'a> {
    pub fn new(ast: &'a mut [Node]) -> Self {
        Analyzer {
            ast,
        }
    }

    pub fn go(&mut self) -> Result<(), Error> {
        self.verify_top_level_decls()?;
        Ok(())
    }

    fn verify_top_level_decls(&mut self) -> Result<(), Error> {
        for decl in self.ast {
            match decl {
                Node::ProcStatement { .. } | Node::ConstStatement { .. } => (),
                n => return Err(Error::InvalidAtTopLevel { node: n.clone() }),
            }
        }
        Ok(())
    }
}
