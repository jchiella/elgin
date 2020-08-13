//! LLVM code generation based on the Elgin IR format

extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::prelude::*;

use std::collections::HashMap;
use std::ffi::{CStr, CString};

use crate::ir::{CompareType, IRProc, Instruction, InstructionType};
use crate::types::Type;
use crate::errors::Span;

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
                    if let InstructionType::Label(label) = ins.contents.ins {
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
                for (i, name) in proc.args.iter().enumerate() {
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

    fn ins(&mut self, ins: &Span<Instruction>) {
        use crate::ir::InstructionType::*;
        let typ = ins.contents.typ.clone();
        match ins.clone().contents.ins {
            Push(s) => self.push(s, typ),
            Load(s) => self.load(s, typ),
            Store(s) => self.store(s, typ),
            Allocate(s) => self.allocate(s, typ),

            Branch(b, e) => self.branch(b, e),
            Jump(l) => self.jump(l),
            Label(l) => self.label(l),

            Call(pn) => self.call(pn),
            Return => self.return_(typ),

            Negate(wrap) => self.negate(typ, wrap),
            Add(wrap) => self.add(typ, wrap),
            Subtract(wrap) => self.subtract(typ, wrap),
            Multiply(wrap) => self.multiply(typ, wrap),
            IntDivide => self.int_divide(typ),

            Divide => self.divide(typ),

            Compare(m) => self.compare(m, typ),
        }
    }

    fn push(&mut self, s: String, typ: Type) {
        unsafe {
            let obj = match typ {
                Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::I128 => {
                    LLVMConstInt(self.llvm_type(&typ), s.parse().unwrap(), 0)
                }
                Type::N8
                | Type::N16
                | Type::N32
                | Type::N64
                | Type::N128 => {
                    LLVMConstInt(self.llvm_type(&typ), s.parse().unwrap(), 0)
                }
                Type::F32
                | Type::F64
                | Type::F128 => {
                    LLVMConstReal(self.llvm_type(&typ), s.parse().unwrap())
                }
                Type::Undefined => {
                    LLVMGetUndef(self.llvm_type(&Type::I8))
                }
                Type::StrLiteral => {
                    LLVMBuildGlobalStringPtr(self.builder, self.cstr(&s), self.cstr("tmpstr"))
                }
                t => todo!("{:?}", t),
            };
            self.stack.push(obj);
        }
    }

    fn load(&mut self, s: String, typ: Type) {
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

    fn store(&mut self, s: String, _typ: Type) {
        let var = self.lookup.get(&s).unwrap();
        unsafe {
            LLVMBuildStore(
                self.builder,
                self.stack.pop().unwrap(),
                *var,
            );
        }
    }

    fn allocate(&mut self, s: String, typ: Type) {
        unsafe {
            let name = self.cstr(&s);
            let alloca = LLVMBuildAlloca(self.builder, self.llvm_type(&typ), name);
            self.lookup.insert(s.clone(), alloca);
            let val = self.stack.pop().unwrap();
            if LLVMIsUndef(val) == 0 {
                LLVMBuildStore(self.builder, val, alloca);
            } else {
                LLVMBuildStore(self.builder, LLVMGetUndef(self.llvm_type(&typ)), alloca);
            }
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

    fn return_(&mut self, typ: Type) {
        unsafe {
            if let Type::Undefined = dbg!(typ) {
                LLVMBuildRetVoid(self.builder);
            } else {
                LLVMBuildRet(self.builder, dbg!(self.stack.pop().unwrap()));
            }
        }
    }

    fn negate(&mut self, typ: Type, wrap: bool) {
        unsafe {
            let neg = match typ {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 => {
                    if wrap {
                        LLVMBuildNeg(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.cstr("tmpneg"),
                        )
                    } else {
                        LLVMBuildNSWNeg(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.cstr("tmpneg"),
                        )
                    }
                },
                Type::N8 | Type::N16 | Type::N32 | Type::N64 | Type::N128 => {
                    if wrap {
                        LLVMBuildNeg(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.cstr("tmpneg"),
                        )
                    } else {
                        LLVMBuildNUWNeg(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.cstr("tmpneg"),
                        )
                    }
                },
                Type::F32
                    | Type::F64
                    | Type::F128 => LLVMBuildFNeg(
                        self.builder,
                        self.stack.pop().unwrap(),
                        self.cstr("tmpneg"),
                ),
                _ => unreachable!(),
            };
            self.stack.push(neg);
        }
    }

    fn add(&mut self, typ: Type, wrap: bool) {
        unsafe {
            let add = match typ {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 => {
                    if wrap {
                        LLVMBuildAdd(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpadd"),
                        )
                    } else {
                        LLVMBuildNSWAdd(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpadd"),
                        )
                    }
                },
                Type::N8 | Type::N16 | Type::N32 | Type::N64 | Type::N128 => {
                    if wrap {
                        LLVMBuildAdd(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpadd"),
                        )
                    } else {
                        LLVMBuildNUWAdd(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpadd"),
                        )
                    }
                },
                Type::F32
                    | Type::F64
                    | Type::F128 => LLVMBuildFAdd(
                        self.builder,
                        self.stack.pop().unwrap(),
                        self.stack.pop().unwrap(),
                        self.cstr("tmpadd"),
                ),
                _ => unreachable!(),
            };
            self.stack.push(add);
        }
    }

    fn subtract(&mut self, typ: Type, wrap: bool) {
        unsafe {
            let v1 = self.stack.pop().unwrap();
            let v2 = self.stack.pop().unwrap();
            let sub = match typ {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 => {
                    if wrap {
                        LLVMBuildSub(
                                self.builder,
                                v2,
                                v1,
                                self.cstr("tmpsub"),
                        )
                    } else {
                        LLVMBuildNSWSub(
                                self.builder,
                                v2,
                                v1,
                                self.cstr("tmpsub"),
                        )
                    }
                },
                Type::N8 | Type::N16 | Type::N32 | Type::N64 | Type::N128 => {
                    if wrap {
                        LLVMBuildSub(
                                self.builder,
                                v2,
                                v1,
                                self.cstr("tmpsub"),
                        )
                    } else {
                        LLVMBuildNUWSub(
                                self.builder,
                                v2,
                                v1,
                                self.cstr("tmpsub"),
                        )
                    }
                },
                Type::F32
                    | Type::F64
                    | Type::F128 => LLVMBuildFSub(
                        self.builder,
                        v2,
                        v1,
                        self.cstr("tmpsub"),
                ),
                _ => unreachable!(),
            };
            self.stack.push(sub);
        }
    }

    fn multiply(&mut self, typ: Type, wrap: bool) {
        unsafe {
            let mul = match typ {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 => {
                    if wrap {
                        LLVMBuildMul(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpmul"),
                        )
                    } else {
                        LLVMBuildNSWMul(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpmul"),
                        )
                    }
                },
                Type::N8 | Type::N16 | Type::N32 | Type::N64 | Type::N128 => {
                    if wrap {
                        LLVMBuildMul(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpmul"),
                        )
                    } else {
                        LLVMBuildNUWMul(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpmul"),
                        )
                    }
                },
                Type::F32
                    | Type::F64
                    | Type::F128 => LLVMBuildFMul(
                        self.builder,
                        self.stack.pop().unwrap(),
                        self.stack.pop().unwrap(),
                        self.cstr("tmpmul"),
                ),
                _ => unreachable!(),
            };
            self.stack.push(mul);
        }
    }

    fn int_divide(&mut self, typ: Type) {
        unsafe {
            let mul = match typ {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 => {
                    LLVMBuildSDiv(
                            self.builder,
                            self.stack.pop().unwrap(),
                            self.stack.pop().unwrap(),
                            self.cstr("tmpdiv"),
                    )
                },
                Type::N8 | Type::N16 | Type::N32 | Type::N64 | Type::N128 => {
                    LLVMBuildUDiv(
                            self.builder,
                            self.stack.pop().unwrap(),
                            self.stack.pop().unwrap(),
                            self.cstr("tmpdiv"),
                    )
                },
                Type::F32
                    | Type::F64
                    | Type::F128 => unreachable!(),
                _ => unreachable!(),
            };
            self.stack.push(mul);
        }
    }

    fn divide(&mut self, typ: Type) {
        unsafe {
            let mul = match typ {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 |
                Type::N8 | Type::N16 | Type::N32 | Type::N64 | Type::N128 => unreachable!(),
                Type::F32
                    | Type::F64
                    | Type::F128 => {
                        LLVMBuildFDiv(
                                self.builder,
                                self.stack.pop().unwrap(),
                                self.stack.pop().unwrap(),
                                self.cstr("tmpdiv"),
                        )
                    },
                _ => unreachable!(),
            };
            self.stack.push(mul);
        }
    }

    fn compare(&mut self, comptype: CompareType, typ: Type) {
        unsafe {
            use llvm::LLVMIntPredicate::*;
            use llvm::LLVMRealPredicate::*;
            let cmp = match typ {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 => {
                    LLVMBuildICmp(
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
                    )
                },
                Type::N8 | Type::N16 | Type::N32 | Type::N64 | Type::N128 => {
                    LLVMBuildICmp(
                        self.builder,
                        match comptype {
                            CompareType::EQ => LLVMIntEQ,
                            CompareType::NE => LLVMIntNE,
                            CompareType::LT => LLVMIntUGT,
                            CompareType::GT => LLVMIntUGT,
                            CompareType::LE => LLVMIntULE,
                            CompareType::GE => LLVMIntUGE,
                        },
                        self.stack.pop().unwrap(),
                        self.stack.pop().unwrap(),
                        self.cstr("tmpcmp"),
                    )
                },
                Type::F32
                    | Type::F64
                    | Type::F128 => {
                        LLVMBuildFCmp(
                            self.builder,
                            match comptype {
                                CompareType::EQ => LLVMRealOEQ,
                                CompareType::NE => LLVMRealONE,
                                CompareType::LT => LLVMRealOGT,
                                CompareType::GT => LLVMRealOGT,
                                CompareType::LE => LLVMRealOLE,
                                CompareType::GE => LLVMRealOGE,
                            },
                            self.stack.pop().unwrap(),
                            self.stack.pop().unwrap(),
                            self.cstr("tmpcmp"),
                        )
                    }
                _ => unreachable!(),
            };
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

    fn llvm_type(&self, t: &Type) -> LLVMTypeRef {
        unsafe {
            match t {
                Type::I8 => LLVMInt8TypeInContext(self.context),
                Type::I16 => LLVMInt16TypeInContext(self.context),
                Type::I32 => LLVMInt32TypeInContext(self.context),
                Type::I64 => LLVMInt64TypeInContext(self.context),
                Type::I128 => LLVMInt128TypeInContext(self.context),

                Type::N8 => LLVMInt8TypeInContext(self.context),
                Type::N16 => LLVMInt16TypeInContext(self.context),
                Type::N32 => LLVMInt32TypeInContext(self.context),
                Type::N64 => LLVMInt64TypeInContext(self.context),
                Type::N128 => LLVMInt128TypeInContext(self.context),

                Type::F32 => LLVMFloatTypeInContext(self.context),
                Type::F64 => LLVMFloatTypeInContext(self.context),
                Type::F128 => LLVMFloatTypeInContext(self.context),

                Type::Bool => LLVMInt1TypeInContext(self.context),

                Type::Ptr(t) => LLVMPointerType(self.llvm_type(&t), 0),
                Type::Array(size, t) => LLVMArrayType(self.llvm_type(&t), *size as u32),

                Type::Undefined => LLVMVoidTypeInContext(self.context),
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
