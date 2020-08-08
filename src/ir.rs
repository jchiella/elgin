//! The Elgin IR creation engine
//! Elgin IR is the intermediate representation which is then used for type analysis in analysis.rs
//! It is then converted into LLVM IR in the codegen phase

use crate::errors::Error;
use crate::parser::{Node, Type};

use std::collections::HashMap;

type IRResult = Result<Vec<Instruction>, Error>;

type Scope = HashMap<String, IRType>;

pub struct IRBuilder<'i> {
    ast: &'i [Node],
    available_type_var: usize,
    scopes: Vec<Scope>,
}

#[derive(Debug)]
pub struct IRProc {
    pub name: String,
    pub arg_types: Vec<IRType>,
    pub ret_type: IRType,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum InstructionType {
    Push(String), // pushes an immediate value to the stack
    Load(String), // pushes a variable's contents to the stack
    Store(String), // pops a value and stores it in variable
    Allocate(String), // creates a new local variable

    Return,

    Negate,
    Add,
    Subtract,
    Multiply,
}

#[derive(Debug, Clone)]
pub enum IRTraits {
    Integral,
    Floating,
    Numeric,
}


#[derive(Debug, Clone)]
pub enum IRType {
    Primitive(Type),
    Variable(usize, Vec<IRTraits>),
    Unknown,

    Undefined,
    NoReturn,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub ins: InstructionType,
    pub typ: IRType,
    pub lineno: usize,
    pub start: usize,
    pub end: usize,
}

impl<'i> IRBuilder<'i> {
    pub fn new(ast: &'i [Node]) -> Self {
        IRBuilder {
            ast,
            available_type_var: 0,
            scopes: vec![],
        }
    }

    pub fn go(&mut self) -> Result<Vec<IRProc>, Error> {
        let mut procs = vec![];
        for node in self.ast {
            match node.clone() {
                Node::ConstStatement { name, typ, value, lineno, start, end, } => {
                    self.const_statement(name, typ, value, lineno, start, end)?;
                },
                Node::ProcStatement { name, args, arg_types, ret_type, body, lineno, start, end, } => {
                    procs.push(self.proc_statement(name, args, arg_types, ret_type, body, lineno, start, end)?)
                },
                n => return Err(Error::InvalidAtTopLevel { node: n }),
            }
        }
        Ok(procs)
    }

    fn node(&mut self, node: &Node) -> IRResult {
        use crate::parser::Node::*;
        Ok(match node.clone() {
            Literal { typ, value, lineno, start, end, } => self.literal(typ, value, lineno, start, end)?,
            Call { name, args, lineno, start, end, } => self.call(name, args, lineno, start, end)?,
            InfixOp { op, left, right, lineno, start, end, } => self.infix_op(op, left, right, lineno, start, end)?,
            PrefixOp { op, right, lineno, start, end, } => self.prefix_op(op, right, lineno, start, end)?,
            PostfixOp { op, left, lineno, start, end, } => self.postfix_op(op, left, lineno, start, end)?,
            IndexOp { object, index, lineno, start, end, } => self.index_op(object, index, lineno, start, end)?,
            VariableRef { name, lineno, start, end, } => self.variable_ref(name, lineno, start, end)?,
            IfStatement { condition, body, else_body, lineno, start, end, } => self.if_statement(condition, body, else_body, lineno, start, end)?,
            WhileStatement { condition, body, lineno, start, end, } => self.while_statement(condition, body, lineno, start, end)?,
            Block { nodes, lineno, start, end, } => self.block(nodes, lineno, start, end)?,
            VarStatement { name, typ, value, lineno, start, end, } => self.var_statement(name, typ, value, lineno, start, end)?,
            ConstStatement { name, typ, value, lineno, start, end, } => self.const_statement(name, typ, value, lineno, start, end)?,
            AssignStatement { name, value, lineno, start, end, } => self.assign_statement(name, value, lineno, start, end)?,
            _ => unreachable!(),
        })
    }

    fn literal(&mut self, 
               typ: Type, 
               value: String, 
               lineno: usize, 
               start: usize, 
               end: usize) -> IRResult {
        Ok(vec![
            Instruction {
                ins: InstructionType::Push(value),
                typ: self.parse_to_ir_type(&typ),
                lineno, start, end,
            }
        ])
    }

    fn call(&mut self, 
            name: String, 
            args: Vec<Node>, 
            lineno: usize, 
            start: usize, 
            end: usize) -> IRResult {
        todo!()
    }

    fn infix_op(&mut self, 
                op: String, 
                left: Box<Node>, 
                right: Box<Node>, 
                lineno: usize, 
                start: usize, 
                end: usize) -> IRResult {
        let mut res = vec![];
        res.append(&mut self.node(&left)?);
        res.append(&mut self.node(&right)?);
        res.push(Instruction {
            ins: match op.as_str() {
                "+" => InstructionType::Add,
                "-" => InstructionType::Subtract,
                "*" => InstructionType::Multiply,
                _ => todo!(),
            },
            typ: IRType::Variable(self.next_type_var(), vec![IRTraits::Numeric]),
            lineno, start, end,
        });
        Ok(res)
    }

    fn prefix_op(&mut self, 
                 op: String, 
                 right: Box<Node>, 
                 lineno: usize, 
                 start: usize, 
                 end: usize) -> IRResult {
        let mut res = vec![];
        res.append(&mut self.node(&right)?);
        res.push(Instruction {
            ins: match op.as_str() {
                "-" => InstructionType::Negate,
                _ => todo!(),
            },
            typ: IRType::Variable(self.next_type_var(), vec![IRTraits::Numeric]),
            lineno, start, end,
        });
        Ok(res)
    }

    fn postfix_op(&mut self, 
                  op: String, 
                  left: Box<Node>, 
                  lineno: usize, 
                  start: usize, 
                  end: usize) -> IRResult {
        todo!()
    }

    fn index_op(&mut self, 
                object: Box<Node>, 
                index: Box<Node>, 
                lineno: usize, 
                start: usize, 
                end: usize) -> IRResult {
        todo!()
    }

    fn variable_ref(&mut self, 
                    name: String, 
                    lineno: usize, 
                    start: usize, 
                    end: usize) -> IRResult {
        let typ = self.locate_var(&name)?;
        Ok(vec![
            Instruction {
                ins: InstructionType::Load(name),
                typ,
                lineno, start, end,
            }
        ])
    }

    fn if_statement(&mut self, 
                    condition: Box<Node>, 
                    body: Box<Node>, 
                    else_body: Box<Node>, 
                    lineno: usize, 
                    start: usize, 
                    end: usize) -> IRResult {
        todo!()
    }

    fn while_statement(&mut self, 
                       condition: Box<Node>, 
                       body: Box<Node>, 
                       lineno: usize, 
                       start: usize, 
                       end: usize) -> IRResult {
        todo!()
    }

    fn block(&mut self, 
             nodes: Vec<Node>, 
             lineno: usize, 
             start: usize, 
             end: usize) -> IRResult {
        todo!()
    }

    fn var_statement(&mut self, 
                     name: String, 
                     typ: Type, 
                     value: Box<Node>, 
                     lineno: usize, 
                     start: usize, 
                     end: usize) -> IRResult {
        let ir_type = self.parse_to_ir_type(&typ);
        self.scopes.last_mut().unwrap().insert(name.clone(), ir_type);
        let mut res = vec![
            Instruction {
                ins: InstructionType::Allocate(name.clone()),
                typ: self.parse_to_ir_type(&typ),
                lineno, start, end,
            },
        ];
        res.append(&mut self.node(&value)?);
        res.push(Instruction {
            ins: InstructionType::Store(name),
            typ: self.parse_to_ir_type(&typ),
            lineno, start, end,
        });
        Ok(res)
    }

    fn assign_statement(&mut self, 
                        name: String, 
                        value: Box<Node>, 
                        lineno: usize, 
                        start: usize, 
                        end: usize) -> IRResult {
        todo!()
    }

    fn const_statement(&mut self, 
                       name: String, 
                       typ: Type, 
                       value: Box<Node>, 
                       lineno: usize, 
                       start: usize, 
                       end: usize) -> IRResult {
        todo!()
    }

    fn proc_statement(&mut self, 
                      name: String, 
                      args: Vec<String>, 
                      arg_types: Vec<Type>, 
                      ret_type: Type, 
                      body: Box<Node>, 
                      lineno: usize, 
                      start: usize, 
                      end: usize) -> Result<IRProc, Error> {
        let mut ins = vec![];
        self.scopes.push(HashMap::new());
        let ir_arg_types: Vec<_> = arg_types.iter().map(|t| self.parse_to_ir_type(&t)).collect();
        let scope = self.scopes.last_mut().unwrap();
        for (i, arg) in args.iter().enumerate() {
            let t = ir_arg_types[i].clone();
            scope.insert(arg.clone(), t);
        }
        if let Node::Block { nodes, .. } = *body {
            for node in nodes {
                ins.append(&mut self.node(&node)?);
            }
            ins.push(Instruction {
                ins: InstructionType::Return,
                typ: self.parse_to_ir_type(&ret_type),
                lineno, start, end,
            });
            Ok(IRProc {
                name,
                arg_types: arg_types.iter().map(|t| self.parse_to_ir_type(&t)).collect(),
                ret_type: self.parse_to_ir_type(&ret_type),
                body: ins,
            })
        } else {
            panic!()
        }
    }

    fn parse_to_ir_type(&mut self, t: &Type) -> IRType {
        use Type::*;
        match t.clone() {
            ConstInt => IRType::Primitive(Type::I64),//IRType::Variable(self.next_type_var(), vec![IRTraits::Integral]),
            ConstFloat => IRType::Primitive(Type::F64),//IRType::Variable(self.next_type_var(), vec![IRTraits::Floating]),
            ConstStr => todo!(),

            Undefined => IRType::Undefined,
            typ => IRType::Primitive(typ),
        }
    }

    fn next_type_var(&mut self) -> usize {
        self.available_type_var += 1;
        self.available_type_var - 1
    }

    fn locate_var(&mut self, name: &String) -> Result<IRType, Error> {
        let mut scope_index = self.scopes.len() - 1;
        while scope_index >= 0 {
            if let Some(typ) = self.scopes[scope_index].get(name) {
                return Ok(typ.clone())
            } 
            if scope_index == 0 {
                break
            }
            scope_index -= 1
        }

        Err(Error::VarNotInScope { name: name.clone() })
    }
}

