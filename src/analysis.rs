//! The static analysis component of Elgin
//! Does fun stuff like type inference 

use crate::parser::Type;
use crate::ir::*;
use crate::errors::Error;

use std::collections::HashMap;

type Constraints = HashMap<IRType, IRType>;

impl<'i> IRBuilder<'i> {

    pub fn analyze(&mut self) -> Result<(), Error> {
        self.scopes.clear();
        let mut new_procs = Vec::new();
        let mut index = 0;
        while index < self.procs.len() {
            self.scopes.push(HashMap::new());
            let proc = self.procs[index].clone();
            let constraints = self.gen_constraints(&proc)?;
            new_procs.push(self.solve_constraints(&proc, &constraints)?);
            index += 1;
        }
        self.procs = new_procs;
        Ok(())
    }

    fn gen_constraints(&mut self, proc: &IRProc) -> Result<Constraints, Error> {
        use InstructionType::*;
        let mut constraints = HashMap::new();
        let mut stack = vec![];
        for ins in &proc.body {
            match ins.ins.clone() {
                Push(_) => {
                    stack.push(ins.typ.clone());
                },
                Load(var) => {
                    stack.push(self.locate_var(&var)?);
                },
                Allocate(var) => {
                    let content_type = stack.pop().unwrap();
                    let var_type = ins.typ.clone();
                    let scope_index = self.scopes.len() - 1;
                    self.scopes[scope_index].insert(var, var_type.clone());
                    //constraints.insert(var_type, content_type);
                    add_constraint(&mut constraints, var_type, content_type);
                },

                Branch(_, _) => {
                    add_constraint(&mut constraints, stack.pop().unwrap(), IRType::Primitive(Type::Bool));
                },
                Jump(_) => (),
                Label(_) => (),

                Return => {
                    //let type_to_return = stack.pop().unwrap();
                    //let ret_type = ins.typ.clone();
                    //let ret_type2 = proc.ret_type;
                    //constraints.insert(type_to_return, ret_type);
                },

                Negate => {
                    let t1 = stack.pop().unwrap();
                    add_constraint(&mut constraints, t1.clone(), ins.typ.clone());
                },
                Add | Subtract | Multiply => {
                    let t1 = stack.pop().unwrap();
                    let t2 = stack.pop().unwrap();
                    add_constraint(&mut constraints, t1.clone(), t2.clone());
                    add_constraint(&mut constraints, t1.clone(), ins.typ.clone());
                    add_constraint(&mut constraints, t2.clone(), ins.typ.clone());
                },

                Compare(_) => {
                    let t1 = stack.pop().unwrap();
                    let t2 = stack.pop().unwrap();
                    add_constraint(&mut constraints, t1.clone(), t2.clone());
                    add_constraint(&mut constraints, ins.typ.clone(), IRType::Primitive(Type::Bool));
                    stack.push(IRType::Primitive(Type::Bool));
                },
            };
        }
        Ok(constraints)
    }

    fn solve_constraints(&self, proc: &IRProc, constraints: &Constraints) -> Result<IRProc, Error> {
        println!("Generated constraints:");
        for (t1, t2) in constraints {
            println!("{:?} == {:?}", t1, t2);
        }
        println!("------------------------");
        let mut new_body = proc.body.clone();
        let mut new_constraints = constraints.clone();

        //while new_constraints.len() > 0 {
        for _ in 1..4 {
            for (t1, t2) in constraints {
                // set t1 == t2
                new_body = substitute_proc_body(new_body, t1, t2); // replace in the proc
                new_constraints = substitute_constraints(&new_constraints, t1, t2); // replace in the rules
            }
        }

        Ok(dbg!(IRProc {
            name: proc.name.clone(),
            arg_types: proc.arg_types.clone(),
            ret_type: proc.ret_type.clone(),
            body: new_body,
        }))

    }

}

fn substitute_proc_body(body: Vec<Instruction>, t1: &IRType, t2: &IRType) -> Vec<Instruction> {
    let mut new_body = vec![];

    for ins in body {
        new_body.push(Instruction {
            ins: ins.ins,
            typ: if ins.typ.clone() == t1.clone() {
                println!("{:?} => {:?}", t1.clone(), t2.clone());
                t2.clone()
            //} else if ins.typ.clone() == t2.clone() {
            //    t1.clone()
            } else {
                ins.typ
            },
            lineno: ins.lineno,
            start: ins.start,
            end: ins.end,

        });
    }
    new_body
}

fn substitute_constraints(constraints: &Constraints, t1: &IRType, t2: &IRType) -> Constraints {
    let mut new_constraints = HashMap::new();

    for (left, right) in constraints {
        if left == right {
            continue;
        }
        let new_left = if *left == *t1 {
            t2.clone()
        } else {
            left.clone()
        };

        let new_right = if *right == *t1 {
            t2.clone()
        } else {
            right.clone()
        };

        new_constraints.insert(new_left, new_right);
    }

    new_constraints
}


fn add_constraint(constraints: &mut Constraints, t1: IRType, t2: IRType) {
    if let IRType::Variable(_) = t2 {
        constraints.insert(t2, t1);
    } else {
        constraints.insert(t1, t2);
    }
}

