//! LLVM code generation based on the Elgin IR format

extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::prelude::*;

use std::collections::HashMap;
use std::ffi::{CStr, CString};

use crate::ir::{CompareType, IRProc, IRType, Instruction, InstructionType};
use crate::parser::Type;

pub struct Generator<'g> {
    procs: &'g [IRProc],

    context: *mut llvm::LLVMContext,
    builder: *mut llvm::LLVMBuilder,
    module: *mut llvm::LLVMModule,

    strings: Vec<CString>,

    stack: Vec<LLVMValueRef>,
    lookup: HashMap<String, LLVMValueRef>,
    labels: HashMap<usize, LLVMBasicBlockRef>,
    llvm_procs: HashMap<String, LLVMValueRef>,

    current_proc: LLVMValueRef,
}

impl<'g> Generator<'g> {
    pub fn new(procs: &'g [IRProc], module_name: &str, file_name: &str) -> Self {
        let context = unsafe { LLVMContextCreate() };
        let builder = unsafe { LLVMCreateBuilderInContext(context) };
        let module = unsafe {
            LLVMModuleCreateWithNameInContext(module_name.as_bytes().as_ptr() as *const _, context)
        };
        unsafe {
            LLVMSetSourceFileName(
                module,
                file_name.as_bytes().as_ptr() as *const _,
                file_name.len(),
            )
        };

        Generator {
            procs,

            context,
            builder,
            module,

            strings: vec![],

            stack: vec![],
            lookup: HashMap::new(),
            labels: HashMap::new(),
            llvm_procs: HashMap::new(),

            current_proc: 0 as LLVMValueRef,
        }
    }

    pub fn go(&mut self) {
        //self.build_header();
        // Create declarations first
        for proc in self.procs {
            unsafe {
                let mut llvm_arg_types: Vec<_> =
                    proc.arg_types.iter().map(|t| self.llvm_type(&t)).collect();
                let proc_type = LLVMFunctionType(
                    self.llvm_type(&proc.ret_type),
                    llvm_arg_types.as_mut_ptr(),
                    llvm_arg_types.len() as u32,
                    0,
                    );
                let this_proc = LLVMAddFunction(self.module, self.cstr(&proc.name), proc_type);
                self.llvm_procs.insert(proc.name.clone(), this_proc);
            }
        }
        // Then evaluate bodies
        for proc in self.procs {
            unsafe {
                for ins in &proc.body {
                    // index labels before starting
                    if let InstructionType::Label(label) = ins.ins {
                        let mut lbl = "lbl".to_string();
                        lbl.push_str(&label.to_string());
                        let bb = LLVMCreateBasicBlockInContext(self.context, self.cstr(&lbl));
                        self.labels.insert(label, bb);
                    }
                }

                if proc.body.len() == 0 { // this is a declaration, not a definition
                    continue 
                }

                self.current_proc = self.llvm_procs[&proc.name];
                let bb = LLVMAppendBasicBlockInContext(
                    self.context,
                    self.current_proc,
                    self.cstr("entry"),
                );
                LLVMPositionBuilderAtEnd(self.builder, bb);
                for (i, name) in proc.arg_names.iter().enumerate() {
                    self.stack.push(LLVMGetParam(self.current_proc, i as u32));
                    self.allocate(name.clone(), proc.arg_types[i].clone());
                }
            }
            for ins in &proc.body {
                self.ins(&ins.clone());
            }
        }
    }

    fn build_header(&mut self) {
        unsafe {
            let mut puts_arg_types = vec![LLVMPointerType(LLVMInt8Type(), 0)];
            let puts_type = LLVMFunctionType(
                LLVMInt32TypeInContext(self.context),
                puts_arg_types.as_mut_ptr(),
                1,
                0,
            );
            LLVMAddFunction(self.module, self.cstr("puts"), puts_type);

            let mut printf_arg_types = vec![LLVMPointerType(LLVMInt8Type(), 0)];
            let printf_type = LLVMFunctionType(
                LLVMInt32TypeInContext(self.context),
                printf_arg_types.as_mut_ptr(),
                1,
                1,
            );
            LLVMAddFunction(self.module, self.cstr("printf"), printf_type);
        }
    }

    fn ins(&mut self, ins: &Instruction) {
        use crate::ir::InstructionType::*;
        let typ = ins.typ.clone();
        match ins.clone().ins {
            Push(s) => self.push(s, typ),
            Load(s) => self.load(s, typ),
            Allocate(s) => self.allocate(s, typ),

            Branch(b, e) => self.branch(b, e),
            Jump(l) => self.jump(l),
            Label(l) => self.label(l),

            Call(pn) => self.call(pn),
            Return => self.return_(typ),

            Negate => self.negate(typ),
            Add => self.add(typ),
            Subtract => self.subtract(typ),
            Multiply => self.multiply(typ),

            Compare(m) => self.compare(m, typ),
        }
    }

    fn push(&mut self, s: String, typ: IRType) {
        unsafe {
            let obj = match typ {
                IRType::Primitive(Type::I8)
                | IRType::Primitive(Type::I16)
                | IRType::Primitive(Type::I32)
                | IRType::Primitive(Type::I64)
                | IRType::Primitive(Type::I128) => {
                    LLVMConstInt(self.llvm_type(&typ), s.parse().unwrap(), 0)
                }
                IRType::Primitive(Type::N8)
                | IRType::Primitive(Type::N16)
                | IRType::Primitive(Type::N32)
                | IRType::Primitive(Type::N64)
                | IRType::Primitive(Type::N128) => {
                    LLVMConstInt(self.llvm_type(&typ), s.parse().unwrap(), 0)
                }
                IRType::Primitive(Type::F32)
                | IRType::Primitive(Type::F64)
                | IRType::Primitive(Type::F128) => {
                    LLVMConstReal(self.llvm_type(&typ), s.parse().unwrap())
                }
                IRType::Undefined => {
                    LLVMConstInt(self.llvm_type(&IRType::Primitive(Type::I64)), 0xffff, 0) // super temporary
                }
                IRType::Primitive(Type::Str) => {
                    LLVMBuildGlobalStringPtr(self.builder, self.cstr(&s), self.cstr("tmpstr"))
                }
                t => todo!("{:?}", t),
            };
            self.stack.push(obj);
        }
    }

    fn load(&mut self, s: String, typ: IRType) {
        let var = self.lookup.get(&s).unwrap();
        unsafe {
            let ld = LLVMBuildLoad2(
                self.builder,
                self.llvm_type(&typ),
                *var,
                self.cstr("tmpload"),
            );
            self.stack.push(ld);
        }
    }

    fn allocate(&mut self, s: String, typ: IRType) {
        unsafe {
            let name = self.cstr(&s);
            let alloca = LLVMBuildAlloca(self.builder, self.llvm_type(&typ), name);
            self.lookup.insert(s.clone(), alloca);
            let val = self.stack.pop().unwrap();
            LLVMBuildStore(self.builder, val, alloca);
        }
    }

    fn call(&mut self, proc_name: String) {
        unsafe {
            let proc = self.llvm_procs[&proc_name];
            let mut args = vec![];
            let arg_count = LLVMCountParams(proc);
            for _ in 0..arg_count {
                args.push(self.stack.pop().unwrap());
            }
            let call = LLVMBuildCall(self.builder, proc, args.as_mut_ptr(), args.len() as u32, self.cstr("tmpcall"));
            self.stack.push(call); 
        }
    }

    fn return_(&mut self, typ: IRType) {
        unsafe {
            if let IRType::Undefined = dbg!(typ) {
                dbg!("Void");
                LLVMBuildRetVoid(self.builder);
            } else {
                LLVMBuildRet(self.builder, dbg!(self.stack.pop().unwrap()));
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
            let add = LLVMBuildAdd(
                self.builder,
                self.stack.pop().unwrap(),
                self.stack.pop().unwrap(),
                self.cstr("tmpadd"),
            );
            self.stack.push(add);
        }
    }

    fn subtract(&mut self, typ: IRType) {
        unsafe {
            let sub = LLVMBuildSub(
                self.builder,
                self.stack.pop().unwrap(),
                self.stack.pop().unwrap(),
                self.cstr("tmpsub"),
            );
            self.stack.push(sub);
        }
    }

    fn multiply(&mut self, typ: IRType) {
        unsafe {
            let mul = LLVMBuildMul(
                self.builder,
                self.stack.pop().unwrap(),
                self.stack.pop().unwrap(),
                self.cstr("tmpmul"),
            );
            self.stack.push(mul);
        }
    }

    fn compare(&mut self, comptype: CompareType, typ: IRType) {
        unsafe {
            use llvm::LLVMIntPredicate::*;
            dbg!(self.stack.clone());
            let cmp = LLVMBuildICmp(
                self.builder,
                match comptype {
                    CompareType::EQ => LLVMIntEQ,
                    CompareType::NE => LLVMIntNE,
                    CompareType::LT => LLVMIntSLT,
                    CompareType::GT => LLVMIntSGT,
                    CompareType::LE => LLVMIntSLE,
                    CompareType::GE => LLVMIntSGE,
                },
                self.stack.pop().unwrap(),
                self.stack.pop().unwrap(),
                self.cstr("tmpcmp"),
            );
            self.stack.push(cmp);
        }
    }

    fn branch(&mut self, then_label: usize, else_label: usize) {
        unsafe {
            let br = LLVMBuildCondBr(
                self.builder,
                self.stack.pop().unwrap(),
                self.labels[&then_label],
                self.labels[&else_label],
            );
            self.stack.push(br);
        }
    }

    fn jump(&mut self, label: usize) {
        unsafe {
            let jmp = LLVMBuildBr(self.builder, self.labels[&label]);
            self.stack.push(jmp);
        }
    }

    fn label(&mut self, label: usize) {
        unsafe {
            LLVMAppendExistingBasicBlock(self.current_proc, self.labels[&label]);
            LLVMPositionBuilderAtEnd(self.builder, self.labels[&label]);
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
                IRType::Primitive(Type::Str) => LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),

                IRType::Undefined => LLVMVoidTypeInContext(self.context),
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
            let mut error_msg: *mut i8 = ""
                .as_bytes()
                .iter()
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
