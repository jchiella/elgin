//! LLVM code generation based on the Elgin IR format

extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::prelude::*;

use std::ffi::{CString, CStr};
use std::collections::HashMap;

use crate::errors::Error;
use crate::parser::Type;
use crate::ir::{Instruction, IRProc, IRType};

type GenResult = Result<LLVMValueRef, Error>;

pub struct Generator<'g> {
    procs: &'g [IRProc], 

    context: *mut llvm::LLVMContext,
    builder: *mut llvm::LLVMBuilder,
    module: *mut llvm::LLVMModule,

    strings: Vec<CString>,

    stack: Vec<LLVMValueRef>,
    lookup: HashMap<String, LLVMValueRef>,
}

impl<'g> Generator<'g> {
    pub fn new(procs: &'g [IRProc], module_name: &str, file_name: &str) -> Self {
        let context = unsafe { LLVMContextCreate() };
        let builder = unsafe { LLVMCreateBuilderInContext(context) };
        let module = unsafe { LLVMModuleCreateWithNameInContext(module_name.as_bytes().as_ptr() as *const _, context) };
        unsafe { LLVMSetSourceFileName(module, file_name.as_bytes().as_ptr() as *const _, file_name.len()) };

        Generator {
            procs,

            context,
            builder,
            module,

            strings: vec![],

            stack: vec![],

            lookup: HashMap::new(),
        }
    }

    pub fn go(&mut self) {
        self.build_header();
        for proc in self.procs {
            unsafe {
                let mut llvm_arg_types: Vec<_> = proc.arg_types.iter().map(|t| self.llvm_type(&t)).collect();
                let proc_type = LLVMFunctionType(LLVMVoidTypeInContext(self.context), llvm_arg_types.as_mut_ptr(), 0, 0);
                let proc = LLVMAddFunction(self.module, self.cstr(&proc.name), proc_type);
                let bb = LLVMAppendBasicBlockInContext(self.context, proc, self.cstr("entry"));
                LLVMPositionBuilderAtEnd(self.builder, bb);
            }
            for ins in &proc.body {
                self.ins(&ins.clone());
            }
        }
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

    fn ins(&mut self, ins: &Instruction) {
        use crate::ir::InstructionType::*;
        let typ = ins.typ.clone();
        match ins.clone().ins {
            Push(s) => self.push(s, typ),
            Load(s) => self.load(s, typ),
            Store(s) => self.store(s, typ),
            Allocate(s) => self.allocate(s, typ),

            Return => self.return_(typ),

            Negate => self.negate(typ),
            Add => self.add(typ),
            Subtract => self.subtract(typ),
            Multiply => self.multiply(typ),
        }
    }

    fn push(&mut self, s: String, typ: IRType) {
        unsafe {
            self.stack.push(match typ {
                IRType::Primitive(Type::I8) 
                    | IRType::Primitive(Type::I16) 
                    | IRType::Primitive(Type::I32) 
                    | IRType::Primitive(Type::I64) 
                    | IRType::Primitive(Type::I128) => LLVMConstInt(self.llvm_type(&typ), s.parse().unwrap(), 0),
                IRType::Primitive(Type::N8) 
                    | IRType::Primitive(Type::N16) 
                    | IRType::Primitive(Type::N32) 
                    | IRType::Primitive(Type::N64) 
                    | IRType::Primitive(Type::N128) => LLVMConstInt(self.llvm_type(&typ), s.parse().unwrap(), 0),
                IRType::Primitive(Type::F32) 
                    | IRType::Primitive(Type::F64) 
                    | IRType::Primitive(Type::F128) => LLVMConstReal(self.llvm_type(&typ), s.parse().unwrap()),
                t => todo!("{:?}", t),
            }) 
        }
    }

    fn load(&mut self, s: String, typ: IRType) {
        let var = self.lookup.get(&s).unwrap();
        unsafe {
            let ld = LLVMBuildLoad2(self.builder, self.llvm_type(&typ), *var, self.cstr("tmpload"));
            self.stack.push(ld);
        }
    }

    fn store(&mut self, s: String, _typ: IRType) {
        let var = self.lookup.get(&s).unwrap();
        unsafe {
            LLVMBuildStore(self.builder, self.stack.pop().unwrap(), *var);
        }
    }

    fn allocate(&mut self, s: String, typ: IRType) {
        unsafe {
            let name = self.cstr(&s);
            let alloca = LLVMBuildAlloca(self.builder, self.llvm_type(&typ), name);
            self.lookup.insert(s, alloca);
        } 
    }

    fn return_(&mut self, typ: IRType) {
        unsafe {
            if let IRType::Undefined = typ {
                LLVMBuildRetVoid(self.builder);
            } else {
                LLVMBuildRet(self.builder, self.stack.pop().unwrap());
            }
        }
    }

    fn negate(&mut self, typ: IRType) {
        unsafe {
            let neg = LLVMBuildNeg(self.builder, self.stack.pop().unwrap(), self.cstr("tmpneg"));
            self.stack.push(neg);
        }
    }

    fn add(&mut self, typ: IRType) {
        unsafe {
            let add = LLVMBuildAdd(self.builder, self.stack.pop().unwrap(), self.stack.pop().unwrap(), self.cstr("tmpadd"));
            self.stack.push(add);
        }
    }

    fn subtract(&mut self, typ: IRType) {
        unsafe {
            let sub = LLVMBuildSub(self.builder, self.stack.pop().unwrap(), self.stack.pop().unwrap(), self.cstr("tmpsub"));
            self.stack.push(sub);
        }
    }

    fn multiply(&mut self, typ: IRType) {
        unsafe {
            let mul = LLVMBuildMul(self.builder, self.stack.pop().unwrap(), self.stack.pop().unwrap(), self.cstr("tmpmul"));
            self.stack.push(mul);
        }
    }

    fn llvm_type(&self, t: &IRType) -> LLVMTypeRef {
        unsafe {
            match t {
                IRType::Primitive(Type::I8) => LLVMInt8TypeInContext(self.context),
                IRType::Primitive(Type::I16) => LLVMInt16TypeInContext(self.context),
                IRType::Primitive(Type::I32) => LLVMInt32TypeInContext(self.context),
                IRType::Primitive(Type::I64) => LLVMInt64TypeInContext(self.context),
                IRType::Primitive(Type::I128) => LLVMInt128TypeInContext(self.context),

                IRType::Primitive(Type::N8) => LLVMInt8TypeInContext(self.context),
                IRType::Primitive(Type::N16) => LLVMInt16TypeInContext(self.context),
                IRType::Primitive(Type::N32) => LLVMInt32TypeInContext(self.context),
                IRType::Primitive(Type::N64) => LLVMInt64TypeInContext(self.context),
                IRType::Primitive(Type::N128) => LLVMInt128TypeInContext(self.context),

                IRType::Primitive(Type::F32) => LLVMFloatTypeInContext(self.context),
                IRType::Primitive(Type::F64) => LLVMFloatTypeInContext(self.context),
                IRType::Primitive(Type::F128) => LLVMFloatTypeInContext(self.context),

                IRType::Primitive(Type::Bool) => LLVMInt1TypeInContext(self.context),
                _ => unreachable!(),
            }
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


