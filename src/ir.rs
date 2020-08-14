//! The Elgin IR creation engine
//! Elgin IR is the intermediate representation which is then used for type analysis in analysis.rs
//! It is then converted into LLVM IR in the codegen phase

use crate::errors::{Logger, Span};
use crate::parser::Node;
use crate::types::Type;

use std::collections::HashMap;
use std::fmt;

type Scope = HashMap<String, Type>;
type IRResult = Option<Vec<Span<Instruction>>>;

pub struct IRBuilder<'i> {
    ast: &'i [Span<Node>],
    pub available_type_var: usize,
    available_label_id: usize,
    pub scopes: Vec<Scope>,
    pub procs: Vec<IRProc>, 
    pub consts: HashMap<String, Span<Node>>,
}

#[derive(Debug, Clone)]
pub struct IRProc {
    pub name: String,
    pub args: Vec<String>,
    pub arg_types: Vec<Type>,
    pub ret_type: Type,
    pub body: Vec<Span<Instruction>>,
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
    IntDivide,
    Divide,

    Compare(CompareType),
}


#[derive(Clone)]
pub struct Instruction {
    pub ins: InstructionType,
    pub typ: Type,
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.ins, self.typ)
    }
}

pub fn spanned(ins: Instruction, pos: usize, len: usize) -> Span<Instruction> {
    Span {
        contents: ins.clone(),
        pos,
        len,
    }
}

impl<'i> IRBuilder<'i> {
    pub fn new(ast: &'i [Span<Node>]) -> Self {
        IRBuilder {
            ast,
            available_type_var: 0,
            available_label_id: 0,
            scopes: vec![],
            procs: vec![],
            consts: HashMap::new(),
        }
    }

    pub fn go(&mut self) -> Option<&Vec<IRProc>> {
        self.build_header();
        // just declare all functions + constants
        for node in self.ast {
            match node.clone().contents {
                Node::ConstStatement {
                    name,
                    typ,
                    value,
                } => {
                    self.const_statement(name, typ, value, node.pos, node.len)?;
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
                n => {
                    Logger::syntax_error(
                        format!("A node of type {:?} is not allowed at the top level of a module", n).as_str(),
                        node.pos,
                        node.len,
                    );
                    return None
                }
            }
        }
        // then actually generate code
        for node in self.ast {
            match node.clone().contents {
                Node::ConstStatement {
                    name,
                    typ,
                    value,
                } => {
                    self.const_statement(name, typ, value, node.pos, node.len);
                }
                Node::ProcStatement {
                    name,
                    args,
                    arg_types,
                    ret_type,
                    body,
                } => {
                    let pstat = self.proc_statement(
                        name, args, arg_types, ret_type, body, node.pos, node.len,
                    )?;
                    // FIXME this is a temporary workaround (procs should really be a hashmap)
                    for (i, proc) in self.procs.iter().enumerate() {
                        if proc.name == pstat.name {
                            self.procs[i] = pstat;
                            break;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        Some(&self.procs)
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

    fn node(&mut self, node: &Span<Node>) -> IRResult { 
        use crate::parser::Node::*;
        Some(match node.clone().contents {
            Literal {
                typ,
                value,
            } => self.literal(typ, value, node.pos, node.len)?,
            Call {
                name,
                args,
            } => self.call(name, args, node.pos, node.len)?,
            InfixOp {
                op,
                left,
                right,
            } => self.infix_op(op, left, right, node.pos, node.len)?,
            PrefixOp {
                op,
                right,
            } => self.prefix_op(op, right, node.pos, node.len)?,
            PostfixOp {
                op,
                left,
            } => self.postfix_op(op, left, node.pos, node.len)?,
            IndexOp {
                object,
                index,
            } => self.index_op(object, index, node.pos, node.len)?,
            VariableRef {
                name,
            } => self.variable_ref(name, node.pos, node.len)?,
            IfStatement {
                condition,
                body,
                else_body,
            } => self.if_statement(condition, body, else_body, node.pos, node.len)?,
            WhileStatement {
                condition,
                body,
            } => self.while_statement(condition, body, node.pos, node.len)?,
            Block {
                nodes,
            } => self.block(nodes, node.pos, node.len)?,
            VarStatement {
                name,
                typ,
                value,
            } => self.var_statement(name, typ, value, node.pos, node.len)?,
            ConstStatement { .. } => {
                Logger::syntax_error("Found const statement not at top level. This feature is NYI.", node.pos, node.len);
                return None;
            },
            AssignStatement {
                name,
                value,
            } => self.assign_statement(name, value, node.pos, node.len)?,
            ReturnStatement {
                val,
            } => self.return_statement(val, node.pos, node.len)?,
            _ => unreachable!(),
        })
    }

    fn literal(
        &mut self,
        typ: Type,
        value: String,
        pos: usize,
        len: usize,
    ) -> IRResult {
        Some(vec![spanned(Instruction {
            ins: InstructionType::Push(value),
            typ,
        }, pos, len)])
    }

    fn call(
        &mut self,
        name: String,
        args: Vec<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        let proc = self.locate_proc(&name)?.clone();
        let mut res = vec![];
        for arg in args {
            res.append(&mut self.node(&arg)?);
        }
        res.push(spanned(Instruction {
            ins: InstructionType::Call(proc.name),
            typ: proc.ret_type,
        }, pos, len));
        Some(res)
    }

    fn infix_op(
        &mut self,
        op: String,
        left: Box<Span<Node>>,
        right: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        let mut res = vec![];
        res.append(&mut self.node(&left)?);
        res.append(&mut self.node(&right)?);

        res.push(spanned(Instruction {
            ins: match op.as_str() {
                "+" => InstructionType::Add(false),
                "-" => InstructionType::Subtract(false),
                "*" => InstructionType::Multiply(false),

                "+~" => InstructionType::Add(true),
                "-~" => InstructionType::Subtract(true),
                "*~" => InstructionType::Multiply(true),

                "//" => InstructionType::IntDivide,
                "/" => InstructionType::Divide,

                "==" => InstructionType::Compare(CompareType::EQ),
                "!=" => InstructionType::Compare(CompareType::NE),
                ">" => InstructionType::Compare(CompareType::GT),
                "<" => InstructionType::Compare(CompareType::LT),
                ">=" => InstructionType::Compare(CompareType::GE),
                "<=" => InstructionType::Compare(CompareType::LE),
                _ => todo!(),
            },
            typ: Type::Variable(self.next_type_var()),
        }, pos, len));
        Some(res)
    }

    fn prefix_op(
        &mut self,
        op: String,
        right: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        let mut res = vec![];
        res.append(&mut self.node(&right)?);
        res.push(spanned(Instruction {
            ins: match op.as_str() {
                "-" => InstructionType::Negate(false),
                "-~" => InstructionType::Negate(true),
                _ => todo!(),
            },
            typ: Type::Variable(self.next_type_var()),
        }, pos, len));
        Some(res)
    }

    fn postfix_op(
        &mut self,
        op: String,
        left: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        todo!("{:?} {:?} {:?} {:?}", op, left, pos, len);
    }

    fn index_op(
        &mut self,
        object: Box<Span<Node>>,
        index: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        todo!("{:?} {:?} {:?} {:?}", object, index, pos, len);
    }

    fn variable_ref(&mut self, name: String, pos: usize, len: usize) -> IRResult {
        if self.consts.contains_key(&name) {
            let constant = self.consts[&name].clone();
            return self.node(&constant);
        }

        let typ = self.locate_var(&name)?;
        Some(vec![spanned(Instruction {
            ins: InstructionType::Load(name),
            typ,
        }, pos, len)])
    }

    fn if_statement(
        &mut self,
        condition: Box<Span<Node>>,
        body: Box<Span<Node>>,
        else_body: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        let mut res = vec![];
        let body_label = self.next_label_id();
        let else_label = self.next_label_id();
        let end_label = self.next_label_id();
        let mut blocks_ending_in_return = 2;

        res.append(&mut self.node(&condition)?);
        res.push(spanned(Instruction {
            ins: InstructionType::Branch(body_label, else_label),
            typ: Type::NoReturn,
        }, pos, len));
        res.push(spanned(Instruction {
            ins: InstructionType::Label(body_label),
            typ: Type::Undefined,
        }, pos, len));
        res.append(&mut self.node(&body)?);
        if res.last().unwrap().contents.ins != InstructionType::Return {
            blocks_ending_in_return -= 1;
            res.push(spanned(Instruction {
                ins: InstructionType::Jump(end_label),
                typ: Type::Undefined,
            }, pos, len));
        }
        res.push(spanned(Instruction {
            ins: InstructionType::Label(else_label),
            typ: Type::Undefined,
        }, pos, len));
        res.append(&mut self.node(&else_body)?);
        if res.last().unwrap().contents.ins != InstructionType::Return {
            blocks_ending_in_return -= 1;
            res.push(spanned(Instruction {
                ins: InstructionType::Jump(end_label),
                typ: Type::Undefined,
            }, pos, len));
        }
        if blocks_ending_in_return < 2 {
            res.push(spanned(Instruction {
                ins: InstructionType::Label(end_label),
                typ: Type::Undefined,
            }, pos, len));
        }
        Some(res)
    }

    fn while_statement(
        &mut self,
        condition: Box<Span<Node>>,
        body: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        let mut res = vec![];
        let cond_label = self.next_label_id();
        let body_label = self.next_label_id();
        let end_label = self.next_label_id();
        let mut blocks_ending_in_return = 1;

        res.push(spanned(Instruction {
            ins: InstructionType::Jump(cond_label),
            typ: Type::Undefined,
        }, pos, len));
        res.push(spanned(Instruction {
            ins: InstructionType::Label(cond_label),
            typ: Type::Undefined,
        }, pos, len));
        res.append(&mut self.node(&condition)?);
        res.push(spanned(Instruction {
            ins: InstructionType::Branch(body_label, end_label),
            typ: Type::NoReturn,
        }, pos, len));
        res.push(spanned(Instruction {
            ins: InstructionType::Label(body_label),
            typ: Type::Undefined,
        }, pos, len));
        res.append(&mut self.node(&body)?);
        if res.last().unwrap().contents.ins != InstructionType::Return {
            blocks_ending_in_return -= 1;
            res.push(spanned(Instruction {
                ins: InstructionType::Jump(cond_label),
                typ: Type::Undefined,
            }, pos, len));
        }
        if blocks_ending_in_return < 2 {
            res.push(spanned(Instruction {
                ins: InstructionType::Label(end_label),
                typ: Type::Undefined,
            }, pos, len));
        }
        Some(res)
    }

    fn block(&mut self, nodes: Vec<Span<Node>>, _pos: usize, _len: usize) -> IRResult {
        let mut res = vec![];
        for node in nodes {
            res.append(&mut self.node(&node)?);
        }
        Some(res)
    }

    fn var_statement(
        &mut self,
        name: String,
        typ: Type,
        value: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.clone(), typ.clone());
        let mut res = self.node(&value)?;
        res.push(spanned(Instruction {
            ins: InstructionType::Allocate(name.clone()),
            typ,
        }, pos, len));
        Some(res)
    }

    fn assign_statement(
        &mut self,
        name: String,
        value: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        let mut res = self.node(&value)?;
        res.push(spanned(Instruction {
            ins: InstructionType::Store(name.clone()),
            typ: self.locate_var(&name)?,
        }, pos, len));
        Some(res)
    }

    fn return_statement(
        &mut self,
        val: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> IRResult {
        let mut res = self.node(&val)?;
        res.push(
            spanned(Instruction {
                ins: InstructionType::Return,
                typ: res.last().unwrap().clone().contents.typ,
            }, pos, len)
        );
        Some(res)
    }

    fn const_statement(
        &mut self,
        name: String,
        _typ: Type,
        value: Box<Span<Node>>,
        _pos: usize,
        _len: usize,
    ) -> Option<()> {
        // TODO: Actual verification that this is a const expression
        self.consts.insert(name, *value.clone());
        Some(())
    }

    fn proc_statement(
        &mut self,
        name: String,
        args: Vec<String>,
        arg_types: Vec<Type>,
        ret_type: Type,
        body: Box<Span<Node>>,
        pos: usize,
        len: usize,
    ) -> Option<IRProc> {
        let mut ins = vec![];
        self.scopes.push(HashMap::new());
        let scope = self.scopes.last_mut().unwrap();
        for (i, arg) in args.iter().enumerate() {
            let t = arg_types[i].clone();
            scope.insert(arg.clone(), t);
        }
        if let Node::Block { nodes, .. } = body.contents {
            for node in &nodes {
                ins.append(&mut self.node(&node)?);
            }
            if ret_type == Type::Undefined && nodes.len() > 0 {
                ins.push(spanned(Instruction {
                    ins: InstructionType::Push("undefined".to_owned()),
                    typ: Type::Undefined,
                }, pos, len));
                ins.push(spanned(Instruction {
                    ins: InstructionType::Return,
                    typ: Type::Undefined,
                }, pos, len));
            }
            Some(IRProc {
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

    pub fn locate_var(&self, name: &String) -> Option<Type> {
        //let mut scope_index = self.scopes.len() - 1;
        //while scope_index >= 0 {
        for scope in self.scopes.iter().rev() {
            if let Some(typ) = scope.get(name) {
                return Some(typ.clone());
            }
            //if scope_index == 0 {
            //    break;
            //}
            //scope_index -= 1
        }

        Logger::name_error(
            format!("Can't find a variable named {} in the current scope", name).as_str(),
            0, 0,
        );
        None
    }

    pub fn locate_proc(&self, name: &String) -> Option<&IRProc> {
        for proc in &self.procs {
            if proc.name == *name {
                return Some(proc);
            }
        }
        Logger::name_error(
            format!("Can't find a procedure named {} in the current module", name).as_str(),
            0, 0,
        );
        None
    }
}
