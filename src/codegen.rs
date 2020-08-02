//! Chi code generation (converts AST to ChiVM IR)

use crate::parser::{Node, Type};
use crate::ir::{Instruction, InstructionSpan};

pub struct Generator<'g> {
    nodes: &'g [Node], 
}

impl<'g> Generator<'g> {
    pub fn new(nodes: &'g [Node]) -> Generator {
        Generator {
            nodes, 
        }
    }

    pub fn go(&mut self) -> Vec<InstructionSpan> {
        let mut ins = Vec::new();
        for node in self.nodes {
            ins.append(&mut self.node(node));
        }
        ins
    }

    fn node(&mut self, node: &Node) -> Vec<InstructionSpan> {
        use crate::parser::Node::*;
        match node.clone() {
            Literal { typ, value, lineno, start, end, } => self.literal(typ, value, lineno, start, end),
            Call { name, args, lineno, start, end, } => self.call(name, args, lineno, start, end),
            InfixOp { op, left, right, lineno, start, end, } => self.infix_op(op, left, right, lineno, start, end),
            PrefixOp { op, right, lineno, start, end, } => self.prefix_op(op, right, lineno, start, end),
            PostfixOp { op, left, lineno, start, end, } => self.postfix_op(op, left, lineno, start, end),
            IndexOp { object, index, lineno, start, end, } => self.index_op(object, index, lineno, start, end),
            VariableRef { name, lineno, start, end, } => self.variable_ref(name, lineno, start, end),
            IfStatement { condition, body, else_body, lineno, start, end, } => self.if_statement(condition, body, else_body, lineno, start, end),
            WhileStatement { condition, body, lineno, start, end, } => self.while_statement(condition, body, lineno, start, end),
            Block { nodes, lineno, start, end, } => self.block(nodes, lineno, start, end),
            LetStatement { name, value, lineno, start, end, } => self.let_statement(name, value, lineno, start, end),
            VarStatement { name, value, lineno, start, end, } => self.var_statement(name, value, lineno, start, end),
            ConstStatement { name, value, lineno, start, end, } => self.const_statement(name, value, lineno, start, end),
            ProcStatement { name, args, body, lineno, start, end, } => self.proc_statement(name, args, body, lineno, start, end),
        }
    }

    fn literal(&mut self, typ: Type, value: String, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        vec![
            InstructionSpan {
                instruction: match typ {
                    Type::Int => Instruction::Push(value),
                    _ => unreachable!(),
                },
                lineno, start, end,
            }
        ]
    }

    fn call(&mut self, name: String, args: Vec<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn infix_op(&mut self, op: String, left: Box<Node>, right: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        let mut ins = Vec::new();
        ins.append(&mut self.node(&*left));
        ins.append(&mut self.node(&*right));
        ins.append(&mut vec![
            InstructionSpan {
                instruction: match op.as_str() {
                    "+" => Instruction::Add,
                    "-" => Instruction::Sub,
                    "*" => Instruction::Mul,
                    "/" => Instruction::Div,
                    _ => unreachable!(),
                },
                lineno, start, end,
            },
        ]);
        ins
    }

    fn prefix_op(&mut self, op: String, right: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        let mut ins = Vec::new(); 
        ins.append(&mut self.node(&*right));
        ins.append(&mut vec![
            InstructionSpan {
                instruction: match op.as_str() {
                    "+" => Instruction::Pos,
                    "-" => Instruction::Neg,
                    _ => unreachable!(),
                },
                lineno, start, end,
            },
        ]);
        ins
    }

    fn postfix_op(&mut self, op: String, left: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        let mut ins = Vec::new();
        ins.append(&mut self.node(&*left));
        ins.append(&mut vec![
            InstructionSpan {
                instruction: match op.as_str() {
                    _ => unreachable!(),
                },
                lineno, start, end,
            },
        ]);
        ins
    }

    fn index_op(&mut self, object: Box<Node>, index: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn variable_ref(&mut self, name: String, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn if_statement(&mut self, condition: Box<Node>, body: Box<Node>, else_body: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn while_statement(&mut self, condition: Box<Node>, body: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn block(&mut self, nodes: Vec<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn let_statement(&mut self, name: String, value: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn var_statement(&mut self, name: String, value: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn const_statement(&mut self, name: String, value: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }

    fn proc_statement(&mut self, name: String, args: Vec<String>, body: Box<Node>, lineno: usize, start: usize, end: usize) -> Vec<InstructionSpan> {
        todo!()
    }
}
