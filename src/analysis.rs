//! The static analysis component of Elgin
//! Does fun stuff like type inference 

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
                    constraints.insert(var_type, content_type);
                },

                Return => {
                    //let type_to_return = stack.pop().unwrap();
                    //let ret_type = ins.typ.clone();
                    //let ret_type2 = proc.ret_type;
                    //constraints.insert(type_to_return, ret_type);
                },

                Negate => (),
                Add => {
                    let t1 = stack.pop().unwrap();
                    let t2 = stack.pop().unwrap();
                    constraints.insert(t1.clone(), t2.clone());
                    constraints.insert(t1.clone(), ins.typ.clone());
                    constraints.insert(t2.clone(), ins.typ.clone());
                },
                Subtract => (),
                Multiply => (),
            };
        }
        Ok(constraints)
    }

    fn solve_constraints(&self, proc: &IRProc, constraints: &Constraints) -> Result<IRProc, Error> {
        println!("Generated constraints:");
        for (t1, t2) in constraints {
            println!("{:?} == {:?}", t1, t2);
        }
        todo!()
    }
}
