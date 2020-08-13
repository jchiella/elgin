//! The static analysis component of Elgin
//! Does fun stuff like type inference

use crate::errors::Error;
use crate::ir::*;
use crate::parser::Type;

use std::collections::HashMap;

//type Constraints = HashMap<Type, Type>;
type Constraints = Vec<(Type, Type)>;

impl<'i> IRBuilder<'i> {
    pub fn analyze(&mut self) -> Result<(), Error> {
        self.scopes.clear();
        let mut new_procs = Vec::new();
        let mut index = 0;
        while index < self.procs.len() {
            self.scopes.push(HashMap::new());
            let scope = self.scopes.last_mut().unwrap();
            for (i, arg_type) in self.procs[index].arg_types.iter().enumerate() {
                scope.insert(self.procs[index].args[i].clone(), arg_type.clone());
            }
            let proc = self.procs[index].clone();
            let constraints = self.gen_constraints(&proc)?;
            new_procs.push(self.solve_constraints(&proc, &constraints)?);
            index += 1;
        }
        self.procs = dbg!(new_procs);
        Ok(())
    }

    fn gen_constraints(&mut self, proc: &IRProc) -> Result<Constraints, Error> {
        use InstructionType::*;
        let mut constraints = Vec::new();
        let mut stack = vec![];
        for ins in &proc.body {
            dbg!(ins.ins.clone());
            match ins.ins.clone() {
                Push(_) => {
                    stack.push(ins.typ.clone());
                }
                Load(var) => {
                    stack.push(self.locate_var(&var)?);
                }
                Store(var) => {
                    let typ = stack.pop().unwrap();
                    add_constraint(&mut constraints, ins.typ.clone(), typ);
                    add_constraint(&mut constraints, ins.typ.clone(), self.locate_var(&var)?);
                }
                Allocate(var) => {
                    let content_type = stack.pop().unwrap();
                    let var_type = ins.typ.clone();
                    let scope_index = self.scopes.len() - 1;
                    self.scopes[scope_index].insert(var, var_type.clone());
                    add_constraint(&mut constraints, var_type, content_type);
                }

                Branch(_, _) => {
                    add_constraint(
                        &mut constraints,
                        stack.pop().unwrap(),
                        Type::Bool,
                    );
                }
                Jump(_) => (),
                Label(_) => (),

                Call(proc_name) => {
                    let proc = self.locate_proc(&proc_name)?;
                    let arg_count = proc.arg_types.len();
                    for t in &proc.arg_types {
                        add_constraint(
                            &mut constraints,
                            stack.remove(stack.len() - arg_count),
                            t.clone(),
                        );
                    }
                    stack.push(proc.ret_type.clone());
                }
                Return => {
                    let type_to_return = stack.pop().unwrap();
                    //let ret_type = ins.typ.clone();
                    add_constraint(&mut constraints, type_to_return, proc.ret_type.clone());
                }

                Negate => {
                    let t1 = stack.pop().unwrap();
                    add_constraint(&mut constraints, t1.clone(), ins.typ.clone());
                }
                Add(_) | Subtract(_) | Multiply(_) => {
                    let t1 = stack.pop().unwrap();
                    let t2 = stack.pop().unwrap();
                    add_constraint(&mut constraints, t1.clone(), t2.clone());
                    add_constraint(&mut constraints, t1.clone(), ins.typ.clone());
                    add_constraint(&mut constraints, t2.clone(), ins.typ.clone());
                    stack.push(ins.typ.clone());
                }

                Compare(_) => {
                    let t1 = stack.pop().unwrap();
                    let t2 = stack.pop().unwrap();
                    add_constraint(&mut constraints, t1.clone(), t2.clone());
                    add_constraint(
                        &mut constraints,
                        ins.typ.clone(),
                        Type::Bool,
                    );
                    stack.push(Type::Bool);
                }
            };
            dbg!(stack.clone());
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
                new_constraints = substitute_constraints(&new_constraints, t1, t2);
                // replace in the rules
            }
        }

        Ok(IRProc {
            name: proc.name.clone(),
            args: proc.args.clone(),
            arg_types: proc.arg_types.clone(),
            ret_type: proc.ret_type.clone(),
            body: new_body,
        })
    }
}

fn substitute_proc_body(body: Vec<Instruction>, t1: &Type, t2: &Type) -> Vec<Instruction> {
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

fn substitute_constraints(constraints: &Constraints, t1: &Type, t2: &Type) -> Constraints {
    let mut new_constraints = Vec::new();

    for (left, right) in constraints {
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

        new_constraints.push((new_left, new_right));
    }

    new_constraints
}

fn add_constraint(constraints: &mut Constraints, t1: Type, t2: Type) {
    println!("Trying to add constraint: {:?} == {:?}", t1.clone(), t2.clone());
    // TODO Some of these constraints just shouldn't be permitted at all and should raise a type
    // error. For example, you shouldn't be able to add a constraint i8 == f64
    if t1 == t2 {
        return;
    }
    if t1 == Type::StrLiteral || t2 == Type::StrLiteral {
        return;
    }
    if t1 == Type::Undefined || t2 == Type::Undefined {
        return;
    }
    if let Type::Variable(_) = t2 {
        constraints.push((t2, t1));
    } else {
        if t2 == Type::IntLiteral
            || t2 == Type::FloatLiteral
            || t2 == Type::StrLiteral {
            constraints.push((t2, t1));
        } else {
            constraints.push((t1, t2));
        }
    }
}
