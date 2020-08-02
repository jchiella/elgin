//! The ChiVM IR module which converts the rust object representation
//! to binary which can be read by the ChiVM

#[derive(Debug)]
pub enum Instruction {
    Push(String),
    Add,
    Sub,
    Mul,
    Div,
    Pos,
    Neg,
}

#[derive(Debug)]
pub struct InstructionSpan {
    pub instruction: Instruction,
    pub lineno: usize,
    pub start: usize,
    pub end: usize,
}
