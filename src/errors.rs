//! Errors

use ErrorType::*;

use std::fmt;
use std::sync::Mutex;

#[derive(Clone)]
pub struct Span<T: fmt::Debug> {
    pub contents: T,
    pub pos: usize,
    pub len: usize,
}

impl<T: fmt::Debug> fmt::Debug for Span<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} @ position {}, of length {}",
            self.contents, self.pos, self.len,
        )
    }
}

#[derive(Debug)]
pub enum ErrorType {
    SyntaxError,
    //TypeError,
    NameError,
}

#[derive(Debug)]
pub struct Error {
    typ: ErrorType,
    msg: String,
    pos: usize,
    len: usize,
}

pub struct Logger {

}


lazy_static! {
    pub static ref ERRORS: Mutex<Vec<Error>> = Mutex::new(vec![]);
}

impl Logger {
    pub fn log(typ: ErrorType, msg: &str, pos: usize, len: usize) {
        ERRORS.lock().unwrap().push(Error {
            typ,
            msg: msg.to_owned(),
            pos,
            len,
        });
    }

    #[inline]
    pub fn name_error(msg: &str, pos: usize, len: usize) {
        Self::log(NameError, msg, pos, len);
    }

    //#[inline]
    //pub fn type_error(msg: &str, pos: usize, len: usize) {
    //    Self::log(TypeError, msg, pos, len);
    //}

    #[inline]
    pub fn syntax_error(msg: &str, pos: usize, len: usize) {
        Self::log(SyntaxError, msg, pos, len);
    }
}
