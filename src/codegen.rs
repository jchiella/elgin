//! Chi code generation (converts AST to ChiVM IR)

extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::prelude::*;

use std::ffi::{CString, CStr};

use crate::errors::Error;
use crate::parser::{Node, Type};

type GenResult = Result<LLVMValueRef, Error>;

pub struct Generator<'g> {
    ast: &'g [Node], 
    context: *mut llvm::LLVMContext,
    builder: *mut llvm::LLVMBuilder,
    module: *mut llvm::LLVMModule,
    strings: Vec<CString>,
}

impl<'g> Generator<'g> {
    pub fn new(ast: &'g [Node], module_name: &str, file_name: &str) -> Self {
        let context = unsafe { LLVMContextCreate() };
        let builder = unsafe { LLVMCreateBuilderInContext(context) };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module_name.as_bytes().as_ptr() as *const _, context) };
        unsafe { LLVMSetSourceFileName(module, file_name.as_bytes().as_ptr() as *const _, file_name.len()) };

        Generator {
            ast,
            context,
            builder,
            module,
            strings: vec![],
        }
    }

    pub fn go(&mut self) -> Result<(), Error> {
        for node in self.ast {
            self.node(node)?;
        }
        Ok(())
    }

    fn node(&mut self, node: &Node) -> GenResult {
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
            ProcStatement { name, args, arg_types, ret_type, body, lineno, start, end, } => self.proc_statement(name, args, arg_types, ret_type, body, lineno, start, end)?,
        })
    }

    fn literal(&mut self, 
               typ: Type, 
               value: String, 
               lineno: usize, 
               start: usize, 
               end: usize) -> GenResult {
        Ok(unsafe { LLVMConstInt(self.llvm_type(&typ), value.parse().unwrap(), 0) })
    }

    fn call(&mut self, 
            name: String, 
            args: Vec<Node>, 
            lineno: usize, 
            start: usize, 
            end: usize) -> GenResult {
        let mut llvm_args = vec![];
        for arg in args {
            llvm_args.push(self.node(&arg)?);
        }
        Ok(unsafe { LLVMBuildCall(self.builder, LLVMGetNamedFunction(self.module, self.cstr(&name)), llvm_args.as_mut_ptr(), llvm_args.len() as u32, self.cstr("tmpcall")) })
    }

    fn infix_op(&mut self, 
                op: String, 
                left: Box<Node>, 
                right: Box<Node>, 
                lineno: usize, 
                start: usize, 
                end: usize) -> GenResult {
        let left_val = self.node(&*left)?;
        let right_val = self.node(&*right)?;
        Ok(unsafe {
            match op.as_str() {
                "+" => LLVMBuildAdd(self.builder, left_val, right_val, self.cstr("tmpadd")),
                "-" => LLVMBuildSub(self.builder, left_val, right_val, self.cstr("tmpsub")),
                "*" => LLVMBuildMul(self.builder, left_val, right_val, self.cstr("tmpmul")),
                "/" => LLVMBuildSDiv(self.builder, left_val, right_val, self.cstr("tmpdiv")),
                _ => todo!(),
            }
        })
    }

    fn prefix_op(&mut self, 
                 op: String, 
                 right: Box<Node>, 
                 lineno: usize, 
                 start: usize, 
                 end: usize) -> GenResult {
        let right_val = self.node(&*right)?;
        Ok(unsafe {
            match op.as_str() {
                "-" => LLVMBuildNeg(self.builder, right_val, self.cstr("tmpneg")),
                _ => todo!(),
            }
        })
    }

    fn postfix_op(&mut self, 
                  op: String, 
                  left: Box<Node>, 
                  lineno: usize, 
                  start: usize, 
                  end: usize) -> GenResult {
        let left_val = self.node(&*left)?;
        Ok(unsafe {
            match op.as_str() {
                _ => todo!(),
            }
        })
    }

    fn index_op(&mut self, 
                object: Box<Node>, 
                index: Box<Node>, 
                lineno: usize, 
                start: usize, 
                end: usize) -> GenResult {
        todo!()
    }

    fn variable_ref(&mut self, 
                    name: String, 
                    lineno: usize, 
                    start: usize, 
                    end: usize) -> GenResult {
        todo!()
    }

    fn if_statement(&mut self, 
                    condition: Box<Node>, 
                    body: Box<Node>, 
                    else_body: Box<Node>, 
                    lineno: usize, 
                    start: usize, 
                    end: usize) -> GenResult {
        todo!()
    }

    fn while_statement(&mut self, 
                       condition: Box<Node>, 
                       body: Box<Node>, 
                       lineno: usize, 
                       start: usize, 
                       end: usize) -> GenResult {
        todo!()
    }

    fn block(&mut self, 
             nodes: Vec<Node>, 
             lineno: usize, 
             start: usize, 
             end: usize) -> GenResult {
        todo!()
    }

    fn var_statement(&mut self, 
                     name: String, 
                     typ: Type, 
                     value: Box<Node>, 
                     lineno: usize, 
                     start: usize, 
                     end: usize) -> GenResult {

        let alloca = unsafe { LLVMBuildAlloca(self.builder, self.llvm_type(&typ), self.cstr(&name)) };
        Ok(unsafe { LLVMBuildStore(self.builder, self.node(&*value)?, alloca) })
    }

    fn const_statement(&mut self, 
                       name: String, 
                       typ: Type, 
                       value: Box<Node>, 
                       lineno: usize, 
                       start: usize, 
                       end: usize) -> GenResult {
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
                      end: usize) -> GenResult {
        let mut param_types: Vec<_> = arg_types.iter().map(|t| self.llvm_type(t)).collect();
        let proc_type = unsafe { LLVMFunctionType(self.llvm_type(&ret_type), param_types.as_mut_ptr(), args.len() as u32, 0) };
        let proc = unsafe { LLVMAddFunction(self.module, self.cstr(&name), proc_type) };
        unsafe {
            let entry_block = LLVMAppendBasicBlockInContext(self.context, proc, self.cstr("entry"));
            LLVMPositionBuilderAtEnd(self.builder, entry_block);
        }
        if let Node::Block{ nodes, .. } = *body {
            for node in nodes {
                self.node(&node)?;
            }
        }
        Ok(proc)
    }

    fn llvm_type(&mut self, t: &Type) -> *mut llvm::LLVMType {
        match t {
            Type::I32 => unsafe { LLVMInt32TypeInContext(self.context) },
            Type::ConstInt => unsafe { LLVMInt32TypeInContext(self.context) },
            Type::Undefined => unsafe { LLVMVoidTypeInContext(self.context) },
            _ => todo!(),
        }
    }

    pub fn to_cstring(&self) -> CString {
        unsafe {
            let llvm_ir_ptr = LLVMPrintModuleToString(self.module);
            let llvm_ir = CStr::from_ptr(llvm_ir_ptr as *const _);

            let module_string = CString::new(llvm_ir.to_bytes()).unwrap();


            LLVMDisposeMessage(llvm_ir_ptr);

            module_string
        }
    }

    pub fn dump_to_file(&mut self, file: &str) -> bool {
        unsafe {
            let mut error_msg: *mut i8 = "".as_bytes().iter()
                .map(|b| *b as i8)
                .collect::<Vec<_>>()
                .as_mut_ptr();
            LLVMPrintModuleToFile(self.module, self.cstr(file), &mut error_msg) == 1
        }
    }

    fn cstr(&mut self, s: &str) -> *const i8 {
        let cstring = CString::new(s).unwrap();
        let ptr = cstring.as_ptr() as *const _;
        self.strings.push(cstring);
        ptr
    }
}

impl<'g> Drop for Generator<'g> {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}


