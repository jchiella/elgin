//! The static analysis component of Elgin
//! Does fun stuff like type inference 

use crate::ir::*;
use crate::errors::Error;

impl<'i> IRBuilder<'i> {

    pub fn analyze(&mut self) -> Result<(), Error> {
        Ok(())
    }
    
}
