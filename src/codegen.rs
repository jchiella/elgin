//! Chi code generation (converts AST to ChiVM IR)

extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::prelude::*;

use std::ffi::{CString, CStr};
use std::collections::HashMap;

use crate::errors::Error;
use crate::parser::{Node, Type};

type GenResult = Result<LLVMValueRef, Error>;

pub struct Generator<'g> {
    ast: &'g [Node], 

    context: *mut llvm::LLVMContext,
    builder: *mut llvm::LLVMBuilder,
    module: *mut llvm::LLVMModule,

    strings: Vec<CString>,

    scope: HashMap<String, LLVMValueRef>,
    constants: HashMap<String, LLVMValueRef>,

    last_basic_block: LLVMBasicBlockRef,
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

            scope: HashMap::new(),
            constants: HashMap::new(),

            last_basic_block: unsafe { 0 as LLVMBasicBlockRef },
        }
    }

    pub fn go(&mut self) -> Result<(), Error> {
        self.build_header();
        for node in self.ast {
            self.node(node)?;
        }
        Ok(())
    }

    fn build_header(&mut self) {
        unsafe {
            let mut puts_arg_types = vec![LLVMPointerType(LLVMInt8Type(), 0)];
            let puts_type = LLVMFunctionType(LLVMInt32TypeInContext(self.context), puts_arg_types.as_mut_ptr(), 1, 0);
            LLVMAddFunction(self.module, self.cstr("puts"), puts_type);
            
            let mut printf_arg_types = vec![LLVMPointerType(LLVMInt8Type(), 0)];
            let printf_type = LLVMFunctionType(LLVMInt32TypeInContext(self.context), printf_arg_types.as_mut_ptr(), 1, 1);
            LLVMAddFunction(self.module, self.cstr("printf"), printf_type);
        }
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
            AssignStatement { name, value, lineno, start, end, } => self.assign_statement(name, value, lineno, start, end)?,
            ProcStatement { name, args, arg_types, ret_type, body, lineno, start, end, } => self.proc_statement(name, args, arg_types, ret_type, body, lineno, start, end)?,
        })
    }

    fn literal(&mut self, 
               typ: Type, 
               value: String, 
               lineno: usize, 
               start: usize, 
               end: usize) -> GenResult {
        Ok(match typ {
            Type::ConstInt => unsafe { LLVMConstInt(self.llvm_type(&typ), value.parse().unwrap(), 0) },
            Type::I32 => unsafe { LLVMConstInt(self.llvm_type(&typ), value.parse().unwrap(), 0) },
            Type::ConstStr => unsafe { LLVMBuildGlobalStringPtr(self.builder, self.cstr(&value), self.cstr("tmpstr")) },
            Type::Str => unsafe { LLVMBuildGlobalStringPtr(self.builder, self.cstr(&value), self.cstr("tmpstr")) },
            _ => todo!(),
        })
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
        Ok(unsafe { 
            let proc = LLVMGetNamedFunction(self.module, self.cstr(&name));
            if LLVMGetTypeKind(LLVMGetReturnType(LLVMGetCalledFunctionType(proc))) != llvm::LLVMTypeKind::LLVMVoidTypeKind {
                LLVMBuildCall(self.builder, LLVMGetNamedFunction(self.module, self.cstr(&name)), llvm_args.as_mut_ptr(), llvm_args.len() as u32, self.cstr("tmpcall")) 
            } else {
                LLVMBuildCall(self.builder, LLVMGetNamedFunction(self.module, self.cstr(&name)), llvm_args.as_mut_ptr(), llvm_args.len() as u32, 0 as *const i8) 
            }
        })
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
        use llvm::LLVMIntPredicate::*;
        Ok(unsafe {
            match op.as_str() {
                "+" => LLVMBuildAdd(self.builder, left_val, right_val, self.cstr("tmpadd")),
                "-" => LLVMBuildSub(self.builder, left_val, right_val, self.cstr("tmpsub")),
                "*" => LLVMBuildMul(self.builder, left_val, right_val, self.cstr("tmpmul")),
                "/" => LLVMBuildSDiv(self.builder, left_val, right_val, self.cstr("tmpdiv")),
                "==" => LLVMBuildICmp(self.builder, LLVMIntEQ, left_val, right_val, self.cstr("tmpeq")),
                "!=" => LLVMBuildICmp(self.builder, LLVMIntNE, left_val, right_val, self.cstr("tmpne")),
                "<" => LLVMBuildICmp(self.builder, LLVMIntSLT, left_val, right_val, self.cstr("tmplt")),
                ">" => LLVMBuildICmp(self.builder, LLVMIntSGT, left_val, right_val, self.cstr("tmpgt")),
                "<=" => LLVMBuildICmp(self.builder, LLVMIntSLE, left_val, right_val, self.cstr("tmple")),
                ">=" => LLVMBuildICmp(self.builder, LLVMIntSGE, left_val, right_val, self.cstr("tmpge")),
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
                "!" => LLVMBuildNot(self.builder, right_val, self.cstr("tmpnot")),
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
        Ok(unsafe { 
            if let Some(_) = self.scope.get(&name) {
                LLVMBuildLoad(self.builder, self.scope[&name], self.cstr("tmpload"))
            } else {
                self.constants[&name]
            }
        })
    }

    fn if_statement(&mut self, 
                    condition: Box<Node>, 
                    body: Box<Node>, 
                    else_body: Box<Node>, 
                    lineno: usize, 
                    start: usize, 
                    end: usize) -> GenResult {
        let cond = self.node(&condition)?;
        let body_block = unsafe { LLVMInsertBasicBlockInContext(self.context, self.last_basic_block, self.cstr("ifbody")) };
        let else_block = unsafe { LLVMInsertBasicBlockInContext(self.context, body_block, self.cstr("ifelse")) };
        let end_block = unsafe { LLVMInsertBasicBlockInContext(self.context, else_block, self.cstr("ifend")) };
        unsafe {
            LLVMMoveBasicBlockAfter(body_block, self.last_basic_block);
            LLVMMoveBasicBlockAfter(else_block, body_block);
            LLVMMoveBasicBlockAfter(end_block, else_block);
        }
        self.last_basic_block = end_block;

        let br = unsafe { LLVMBuildCondBr(self.builder, cond, body_block, else_block) };

        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, body_block);
            self.node(&body)?;
            LLVMBuildBr(self.builder, end_block);
            LLVMPositionBuilderAtEnd(self.builder, else_block);
            self.node(&else_body)?;
            LLVMBuildBr(self.builder, end_block);
            LLVMPositionBuilderAtEnd(self.builder, end_block);
        }

        Ok(br)
    }

    fn while_statement(&mut self, 
                       condition: Box<Node>, 
                       body: Box<Node>, 
                       lineno: usize, 
                       start: usize, 
                       end: usize) -> GenResult {
        let cond_block = unsafe { LLVMInsertBasicBlockInContext(self.context, self.last_basic_block, self.cstr("whilecond")) };
        let body_block = unsafe { LLVMInsertBasicBlockInContext(self.context, cond_block, self.cstr("whilebody")) };
        let end_block = unsafe { LLVMInsertBasicBlockInContext(self.context, body_block, self.cstr("whileend")) };
        unsafe {
            LLVMMoveBasicBlockAfter(cond_block, self.last_basic_block);
            LLVMMoveBasicBlockAfter(body_block, cond_block);
            LLVMMoveBasicBlockAfter(end_block, body_block);
        }
        self.last_basic_block = end_block;
        unsafe { LLVMBuildBr(self.builder, cond_block) };

        unsafe { LLVMPositionBuilderAtEnd(self.builder, cond_block) };
        let cond = self.node(&condition)?;
        let br = unsafe { LLVMBuildCondBr(self.builder, cond, body_block, end_block) };

        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, body_block);
            self.node(&body)?;
            LLVMBuildBr(self.builder, cond_block);
            LLVMPositionBuilderAtEnd(self.builder, end_block);
        }

        Ok(br)
    }

    fn block(&mut self, 
             nodes: Vec<Node>, 
             lineno: usize, 
             start: usize, 
             end: usize) -> GenResult {
        for node in nodes {
            self.node(&node)?;
        }
        Ok(unsafe { LLVMGetUndef(LLVMVoidType()) })
    }

    fn var_statement(&mut self, 
                     name: String, 
                     typ: Type, 
                     value: Box<Node>, 
                     lineno: usize, 
                     start: usize, 
                     end: usize) -> GenResult {

        Ok(match typ {
            Type::Str => panic!(),
            _ => {
                let alloca = unsafe { LLVMBuildAlloca(self.builder, self.llvm_type(&typ), self.cstr(&name)) };
                self.scope.insert(name, alloca);
                unsafe { LLVMBuildStore(self.builder, self.node(&*value)?, alloca) }
            },
        })
    }

    fn assign_statement(&mut self, 
                        name: String, 
                        value: Box<Node>, 
                        lineno: usize, 
                        start: usize, 
                        end: usize) -> GenResult {

        Ok(unsafe { LLVMBuildStore(self.builder, self.node(&*value)?, self.scope[&name]) })
    }

    fn const_statement(&mut self, 
                       name: String, 
                       typ: Type, 
                       value: Box<Node>, 
                       lineno: usize, 
                       start: usize, 
                       end: usize) -> GenResult {
        Ok(match *value {
            Node::Literal { typ: Type::ConstStr, value: v, .. } => {
                let strptr = unsafe { LLVMBuildGlobalStringPtr(self.builder, self.cstr(&v), self.cstr(&name)) };
                self.constants.insert(name, strptr);
                strptr
            },
            Node::Literal { typ: t, value: v, .. } => {
                let global = unsafe { LLVMAddGlobal(self.module, self.llvm_type(&t), self.cstr(&name)) };
                unsafe { LLVMSetGlobalConstant(global, 1) };
                self.constants.insert(name, global);
                global
            },
            _ => panic!(),
        })
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
        self.scope.clear();
        let mut param_types: Vec<_> = arg_types.iter().map(|t| self.llvm_type(t)).collect();
        let proc_type = unsafe { LLVMFunctionType(self.llvm_type(&ret_type), param_types.as_mut_ptr(), args.len() as u32, 0) };
        let proc = unsafe { LLVMAddFunction(self.module, self.cstr(&name), proc_type) };
        unsafe {
            let entry_block = LLVMAppendBasicBlockInContext(self.context, proc, self.cstr("entry"));
            self.last_basic_block = entry_block;
            LLVMPositionBuilderAtEnd(self.builder, entry_block);
        }
        unsafe {
            for (i, arg) in args.iter().enumerate() {
                let alloca_type = self.llvm_type(&arg_types[i]);
                let nm = self.cstr(&arg);
                let alloca = LLVMBuildAlloca(self.builder, alloca_type, nm);
                self.scope.insert(arg.clone(), alloca);
                LLVMBuildStore(self.builder, LLVMGetParam(proc, i as u32), alloca);
            }
        }
        self.node(&body)?;
        if ret_type == Type::Undefined {
            unsafe { LLVMBuildRetVoid(self.builder) };
        } else {
            unsafe { LLVMBuildRet(self.builder, LLVMGetLastInstruction(LLVMGetLastBasicBlock(proc))) };
        }
        Ok(proc)
    }

    fn llvm_type(&mut self, t: &Type) -> *mut llvm::LLVMType {
        match t {
            Type::I32 => unsafe { LLVMInt32TypeInContext(self.context) },
            Type::ConstInt => unsafe { LLVMInt32TypeInContext(self.context) },
            Type::Undefined => unsafe { LLVMVoidTypeInContext(self.context) },
            Type::Str => unsafe { LLVMPointerType(LLVMInt8TypeInContext(self.context), 0) },
            Type::ConstStr => unsafe { LLVMPointerType(LLVMInt8TypeInContext(self.context), 0) },
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


