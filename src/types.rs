//! Types, types, and more types...

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

    Undefined,
    NoReturn,

    Ptr(Box<Type>),

    Array(usize, Box<Type>),
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
            Array(size, t) => write!(f, "[{}]{:?}", size, t),

            Variable(n) => write!(f, "${}", n),

            Undefined => write!(f, "undefined"),
            NoReturn => write!(f, "noreturn"),
        }?;
        Ok(())
    }
}

