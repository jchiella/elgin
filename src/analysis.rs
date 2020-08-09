//! The static analysis component of Elgin
//! Does fun tests like type inference 

use crate::parser::Type;
use crate::ir::{
    IRProc,
    IRType,
    IRTraits,
    InstructionType,
};
use crate::errors::Error;

use std::fmt;

pub struct TypeRule(IRType, IRType);

impl fmt::Debug for TypeRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} == {:?}", self.0, self.1)
    } 
}

pub struct Analyzer<'a> {
    pub procs: &'a mut [IRProc],
    stack: Vec<IRType>,
    pub rules: Vec<TypeRule>,
}

impl<'a> Analyzer<'a> {
    pub fn new(procs: &'a mut [IRProc]) -> Self {
        Analyzer {
            procs,
            stack: vec![],
            rules: vec![],
        }
    }

    pub fn go(&mut self) -> Result<(), Error> {
        let mut proc = self.procs[0].clone();
        self.infer_types(&mut proc)?;
        self.concretify_constants(&mut proc);
        self.apply_rules();
        Ok(())
    }

    fn infer_types(&mut self, proc: &mut IRProc) -> Result<(), Error> {
        use InstructionType::*;
        for ins in proc.body.iter_mut() {
            match ins.clone().ins {
                Push(s) => {
                    self.stack.push(ins.clone().typ);
                },
                Load(s) => {
                    let t1 = ins.clone().typ;
                    let t2 = self.stack.pop().unwrap();
                    if let Some(ty) = self.unify_types(t1, t2) {
                        self.stack.push(ty);
                    } else {
                        return Err(Error::TypeError);
                    }
                },
                Store(s) => {
                    let t1 = ins.clone().typ;
                    let t2 = self.stack.pop().unwrap();
                    if let Some(ty) = self.unify_types(t1, t2) {
                        self.stack.push(ty);
                    } else {
                        return Err(Error::TypeError);
                    }
                },
                Allocate(s) => (),

                Return => (),

                Negate => {
                    let t1 = self.stack.pop().unwrap();
                },
                Add => {
                    let t1 = self.stack.pop().unwrap();
                    let t2 = self.stack.pop().unwrap();
                },
                Subtract => {
                    let t1 = self.stack.pop().unwrap();
                    let t2 = self.stack.pop().unwrap();
                },
                Multiply => {
                    let t1 = self.stack.pop().unwrap();
                    let t2 = self.stack.pop().unwrap();
                },
            }
        }
        Ok(())
    }

    fn apply_rules(&mut self) {
        for rule in &self.rules {
            let TypeRule(replacer, replaced) = rule;
            for proc in self.procs.iter_mut() {
                for ins in proc.body.iter_mut() {
                    if ins.typ == *replacer {
                        ins.typ = replaced.clone();
                    }
                }
            }
        }
    }

    fn unify_types(&mut self, t1: IRType, t2: IRType) -> Option<IRType> {
        Some(match t1.clone() {
            IRType::Primitive(ty1) => {
                match t2.clone() {
                    IRType::Primitive(ty2) if ty1 == ty2 => {
                        println!("Unified {:?} with {:?}", t1, t2);
                        t1
                    },
                    IRType::Variable(..) => {
                        println!("Unified {:?} with {:?}", t1, t2);
                        self.rules.push(TypeRule(t2, t1.clone()));
                        t1
                    },
                    _ => todo!(),
                }
            },
            IRType::Variable(..) => {
                match t2 {
                    IRType::Primitive(_) => {
                        println!("Unified {:?} with {:?}", t1, t2);
                       self.rules.push(TypeRule(t1, t2.clone()));
                       t2
                    },
                    IRType::Variable(..) => {
                        println!("Unified {:?} with {:?}", t1, t2);
                        self.rules.push(TypeRule(t1.clone(), t2));
                        t1
                    },
                    _ => todo!(),
                }
            },
            _ => todo!(),
        })
    }

    fn concretify_constants(&mut self, proc: &mut IRProc) {
        for ins in proc.body.iter_mut() {
            match ins.clone().typ {
                IRType::Variable(_, bounds) if bounds.contains(&IRTraits::Integral) => {
                    println!("Concretified {:?}", ins.typ);
                    self.rules.push(TypeRule(ins.clone().typ, IRType::Primitive(Type::I64)));
                },
                IRType::Variable(_, bounds) if bounds.contains(&IRTraits::Floating) => {
                    println!("Concretified {:?}", ins.typ);
                    self.rules.push(TypeRule(ins.clone().typ, IRType::Primitive(Type::F64)));
                },
                _ => (),
            }
        }
    }
}
