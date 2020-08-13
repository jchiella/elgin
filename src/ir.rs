//! The Elgin IR creation engine
//! Elgin IR is the intermediate representation which is then used for type analysis in analysis.rs
//! It is then converted into LLVM IR in the codegen phase

use crate::errors::Error;
use crate::parser::{Node, Type};

use std::collections::HashMap;
use std::fmt;

type IRResult = Result<Vec<Instruction>, Error>;

type Scope = HashMap<String, Type>;

pub struct IRBuilder<'i> {
    ast: &'i [Node],
    pub available_type_var: usize,
    available_label_id: usize,
    pub scopes: Vec<Scope>,
    pub procs: Vec<IRProc>, }

#[derive(Debug, Clone)]
pub struct IRProc {
    pub name: String,
    pub args: Vec<String>,
    pub arg_types: Vec<Type>,
    pub ret_type: Type,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompareType {
    EQ,
    NE,
    GT,
    LT,
    GE,
    LE,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstructionType {
    Push(String),     // pushes an immediate value to the stack
    Load(String),     // pushes a variable's contents to the stack
    Store(String),    // pops a value from the stack into a variable
    Allocate(String), // creates a new local variable and gives it the top value of the stack

    Branch(usize, usize), // conditional branch with if body and else body
    Jump(usize),          // unconditional jump

    Label(usize), // location for jumps and branches

    Call(String), // call another proc from this one
    Return,       // return to the calling proc with the value on the stack

    Negate(bool), // whether or not wrapping is enabled
    Add(bool), 
    Subtract(bool),
    Multiply(bool),

    Compare(CompareType),
}


#[derive(Clone)]
pub struct Instruction {
    pub ins: InstructionType,
    pub typ: Type,
    pub lineno: usize,
    pub start: usize,
    pub end: usize,
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.ins, self.typ)
    }
}

impl<'i> IRBuilder<'i> {
    pub fn new(ast: &'i [Node]) -> Self {
        IRBuilder {
            ast,
            available_type_var: 0,
            available_label_id: 0,
            scopes: vec![],
            procs: vec![],
        }
    }

    pub fn go(&mut self) -> Result<&Vec<IRProc>, Error> {
        self.build_header();
        // just declare all functions + constants
        for node in self.ast {
            match node.clone() {
                Node::ConstStatement {
                    name,
                    typ,
                    value,
                    lineno,
                    start,
                    end,
                } => {
                    self.const_statement(name, typ, value, lineno, start, end)?;
                }
                Node::ProcStatement {
                    name,
                    args,
                    arg_types,
                    ret_type,
                    ..
                } => {
                    self.procs.push(IRProc {
                        name,
                        args,
                        arg_types,
                        ret_type,
                        body: vec![],
                    });
                }
                n => return Err(Error::InvalidAtTopLevel { node: n }),
            }
        }
        // then actually generate code
        for node in self.ast {
            match node.clone() {
                Node::ConstStatement {
                    name,
                    typ,
                    value,
                    lineno,
                    start,
                    end,
                } => {
                    todo!()
                }
                Node::ProcStatement {
                    name,
                    args,
                    arg_types,
                    ret_type,
                    body,
                    lineno,
                    start,
                    end,
                } => {
                    let pstat = self.proc_statement(
                        name, args, arg_types, ret_type, body, lineno, start, end,
                    )?;
                    // FIXME this is a temporary workaround (procs should really be a hashmap)
                    for (i, proc) in self.procs.iter().enumerate() {
                        if proc.name == pstat.name {
                            self.procs[i] = pstat;
                            break;
                        }
                    }
                }
                n => return Err(Error::InvalidAtTopLevel { node: n }),
            }
        }
        Ok(&self.procs)
    }

    fn build_header(&mut self) {
        self.procs.push(IRProc {
            name: "puts".to_owned(),
            args: vec!["s".to_owned()],
            arg_types: vec![Type::Ptr(Box::new(Type::I8))],
            ret_type: Type::I32,
            body: vec![],
        });
    }

    fn node(&mut self, node: &Node) -> IRResult {
        use crate::parser::Node::*;
        Ok(match node.clone() {
            Literal {
                typ,
                value,
                lineno,
                start,
                end,
            } => self.literal(typ, value, lineno, start, end)?,
            Call {
                name,
                args,
                lineno,
                start,
                end,
            } => self.call(name, args, lineno, start, end)?,
            InfixOp {
                op,
                left,
                right,
                lineno,
                start,
                end,
            } => self.infix_op(op, left, right, lineno, start, end)?,
            PrefixOp {
                op,
                right,
                lineno,
                start,
                end,
            } => self.prefix_op(op, right, lineno, start, end)?,
            PostfixOp {
                op,
                left,
                lineno,
                start,
                end,
            } => self.postfix_op(op, left, lineno, start, end)?,
            IndexOp {
                object,
                index,
                lineno,
                start,
                end,
            } => self.index_op(object, index, lineno, start, end)?,
            VariableRef {
                name,
                lineno,
                start,
                end,
            } => self.variable_ref(name, lineno, start, end)?,
            IfStatement {
                condition,
                body,
                else_body,
                lineno,
                start,
                end,
            } => self.if_statement(condition, body, else_body, lineno, start, end)?,
            WhileStatement {
                condition,
                body,
                lineno,
                start,
                end,
            } => self.while_statement(condition, body, lineno, start, end)?,
            Block {
                nodes,
                lineno,
                start,
                end,
            } => self.block(nodes, lineno, start, end)?,
            VarStatement {
                name,
                typ,
                value,
                lineno,
                start,
                end,
            } => self.var_statement(name, typ, value, lineno, start, end)?,
            ConstStatement {
                name,
                typ,
                value,
                lineno,
                start,
                end,
            } => self.const_statement(name, typ, value, lineno, start, end)?,
            AssignStatement {
                name,
                value,
                lineno,
                start,
                end,
            } => self.assign_statement(name, value, lineno, start, end)?,
            ReturnStatement {
                val,
                lineno,
                start,
                end,
            } => self.return_statement(val, lineno, start, end)?,
            _ => unreachable!(),
        })
    }

    fn literal(
        &mut self,
        typ: Type,
        value: String,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        Ok(vec![Instruction {
            ins: InstructionType::Push(value),
            typ,
            lineno,
            start,
            end,
        }])
    }

    fn call(
        &mut self,
        name: String,
        args: Vec<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        let proc = self.locate_proc(&name)?.clone();
        let mut res = vec![];
        for arg in args {
            res.append(&mut self.node(&arg)?);
        }
        res.push(Instruction {
            ins: InstructionType::Call(proc.name),
            typ: proc.ret_type,
            lineno,
            start,
            end,
        });
        Ok(res)
    }

    fn infix_op(
        &mut self,
        op: String,
        left: Box<Node>,
        right: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        let mut res = vec![];
        res.append(&mut self.node(&left)?);
        res.append(&mut self.node(&right)?);

        res.push(Instruction {
            ins: match op.as_str() {
                "+" => InstructionType::Add(false),
                "-" => InstructionType::Subtract(false),
                "*" => InstructionType::Multiply(false),

                "+~" => InstructionType::Add(true),
                "-~" => InstructionType::Subtract(true),
                "*~" => InstructionType::Multiply(true),

                "==" => InstructionType::Compare(CompareType::EQ),
                "!=" => InstructionType::Compare(CompareType::NE),
                ">" => InstructionType::Compare(CompareType::GT),
                "<" => InstructionType::Compare(CompareType::LT),
                ">=" => InstructionType::Compare(CompareType::GE),
                "<=" => InstructionType::Compare(CompareType::LE),
                _ => todo!(),
            },
            typ: Type::Variable(self.next_type_var()),
            lineno,
            start,
            end,
        });
        Ok(res)
    }

    fn prefix_op(
        &mut self,
        op: String,
        right: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        let mut res = vec![];
        res.append(&mut self.node(&right)?);
        res.push(Instruction {
            ins: match op.as_str() {
                "-" => InstructionType::Negate(false),
                "-~" => InstructionType::Negate(true),
                _ => todo!(),
            },
            typ: Type::Variable(self.next_type_var()),
            lineno,
            start,
            end,
        });
        Ok(res)
    }

    fn postfix_op(
        &mut self,
        op: String,
        left: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        todo!()
    }

    fn index_op(
        &mut self,
        object: Box<Node>,
        index: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        todo!()
    }

    fn variable_ref(&mut self, name: String, lineno: usize, start: usize, end: usize) -> IRResult {
        let typ = self.locate_var(&name)?;
        Ok(vec![Instruction {
            ins: InstructionType::Load(name),
            typ,
            lineno,
            start,
            end,
        }])
    }

    fn if_statement(
        &mut self,
        condition: Box<Node>,
        body: Box<Node>,
        else_body: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        let mut res = vec![];
        let body_label = self.next_label_id();
        let else_label = self.next_label_id();
        let end_label = self.next_label_id();
        let mut blocks_ending_in_return = 2;

        res.append(&mut self.node(&condition)?);
        res.push(Instruction {
            ins: InstructionType::Branch(body_label, else_label),
            typ: Type::NoReturn,
            lineno,
            start,
            end,
        });
        res.push(Instruction {
            ins: InstructionType::Label(body_label),
            typ: Type::Undefined,
            lineno,
            start,
            end,
        });
        res.append(&mut self.node(&body)?);
        if res.last().unwrap().ins != InstructionType::Return {
            blocks_ending_in_return -= 1;
            res.push(Instruction {
                ins: InstructionType::Jump(end_label),
                typ: Type::Undefined,
                lineno,
                start,
                end,
            });
        }
        res.push(Instruction {
            ins: InstructionType::Label(else_label),
            typ: Type::Undefined,
            lineno,
            start,
            end,
        });
        res.append(&mut self.node(&else_body)?);
        if res.last().unwrap().ins != InstructionType::Return {
            blocks_ending_in_return -= 1;
            res.push(Instruction {
                ins: InstructionType::Jump(end_label),
                typ: Type::Undefined,
                lineno,
                start,
                end,
            });
        }
        if blocks_ending_in_return < 2 {
            res.push(Instruction {
                ins: InstructionType::Label(end_label),
                typ: Type::Undefined,
                lineno,
                start,
                end,
            });
        }
        Ok(res)
    }

    fn while_statement(
        &mut self,
        condition: Box<Node>,
        body: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        todo!()
    }

    fn block(&mut self, nodes: Vec<Node>, lineno: usize, start: usize, end: usize) -> IRResult {
        let mut res = vec![];
        for node in nodes {
            res.append(&mut self.node(&node)?);
        }
        Ok(res)
    }

    fn var_statement(
        &mut self,
        name: String,
        typ: Type,
        value: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.clone(), typ.clone());
        let mut res = self.node(&value)?;
        res.push(Instruction {
            ins: InstructionType::Allocate(name.clone()),
            typ,
            lineno,
            start,
            end,
        });
        Ok(res)
    }

    fn assign_statement(
        &mut self,
        name: String,
        value: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        let mut res = self.node(&value)?;
        res.push(Instruction {
            ins: InstructionType::Store(name.clone()),
            typ: self.locate_var(&name)?,
            lineno,
            start,
            end,
        });
        Ok(res)
    }

    fn return_statement(
        &mut self,
        val: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        let mut res = self.node(&val)?;
        res.push(
            Instruction {
                ins: InstructionType::Return,
                typ: res.last().unwrap().clone().typ,
                lineno, start, end,
            }
        );
        Ok(res)
    }

    fn const_statement(
        &mut self,
        name: String,
        typ: Type,
        value: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> IRResult {
        todo!()
    }

    fn proc_statement(
        &mut self,
        name: String,
        args: Vec<String>,
        arg_types: Vec<Type>,
        ret_type: Type,
        body: Box<Node>,
        lineno: usize,
        start: usize,
        end: usize,
    ) -> Result<IRProc, Error> {
        let mut ins = vec![];
        self.scopes.push(HashMap::new());
        let scope = self.scopes.last_mut().unwrap();
        for (i, arg) in args.iter().enumerate() {
            let t = arg_types[i].clone();
            scope.insert(arg.clone(), t);
        }
        if let Node::Block { nodes, .. } = *body {
            for node in &nodes {
                ins.append(&mut self.node(&node)?);
            }
            if ret_type == Type::Undefined && nodes.len() > 0 {
                ins.push(Instruction {
                    ins: InstructionType::Push("undefined".to_owned()),
                    typ: Type::Undefined,
                    lineno,
                    start,
                    end,
                });
                ins.push(Instruction {
                    ins: InstructionType::Return,
                    typ: Type::Undefined,
                    lineno,
                    start,
                    end,
                });
            }
            Ok(IRProc {
                name,
                args,
                arg_types,
                ret_type,
                body: ins,
            })
        } else {
            panic!()
        }
    }

    pub fn next_type_var(&mut self) -> usize {
        self.available_type_var += 1;
        self.available_type_var - 1
    }

    fn next_label_id(&mut self) -> usize {
        self.available_label_id += 1;
        self.available_label_id - 1
    }

    pub fn locate_var(&self, name: &String) -> Result<Type, Error> {
        let mut scope_index = self.scopes.len() - 1;
        while scope_index >= 0 {
            if let Some(typ) = self.scopes[scope_index].get(name) {
                return Ok(typ.clone());
            }
            if scope_index == 0 {
                break;
            }
            scope_index -= 1
        }

        Err(Error::VarNotInScope { name: name.clone() })
    }

    pub fn locate_proc(&self, name: &String) -> Result<&IRProc, Error> {
        for proc in &self.procs {
            if proc.name == *name {
                return Ok(proc);
            }
        }
        Err(Error::NoProc { name: name.clone() })
    }
}
