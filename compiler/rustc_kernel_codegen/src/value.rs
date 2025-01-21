use std::fmt::Display;

use rustc_data_structures::intern::Interned;
use crate::{basic_block::BasicBlock, function::FunctionNVVM, module::{Assemble, ModuleNVVM}, ty::{TyNVVM, TypeNVVM}, GlobalNVVM};

pub trait ToVal<'m> {
    fn to_val(self, module: &mut ModuleNVVM<'m>) -> Val<'m>;
}

pub trait AssembleVal<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>, func: &FunctionNVVM<'m>) -> String;
}

#[derive(Debug)]
pub enum Comp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Sgt,
    Sge,
    Slt,
    Sle,
}

impl From<rustc_codegen_ssa::common::IntPredicate> for Comp {
    fn from(pred: rustc_codegen_ssa::common::IntPredicate) -> Self {
        match pred {
            rustc_codegen_ssa::common::IntPredicate::IntEQ => Comp::Eq,
            rustc_codegen_ssa::common::IntPredicate::IntNE => Comp::Ne,
            rustc_codegen_ssa::common::IntPredicate::IntULT => Comp::Lt,
            rustc_codegen_ssa::common::IntPredicate::IntULE => Comp::Le,
            rustc_codegen_ssa::common::IntPredicate::IntUGT => Comp::Gt,
            rustc_codegen_ssa::common::IntPredicate::IntUGE => Comp::Ge,
            rustc_codegen_ssa::common::IntPredicate::IntSGT => Comp::Sgt,
            rustc_codegen_ssa::common::IntPredicate::IntSGE => Comp::Sge,
            rustc_codegen_ssa::common::IntPredicate::IntSLT => Comp::Slt,
            rustc_codegen_ssa::common::IntPredicate::IntSLE => Comp::Sle,
            
        }
    }
}

impl Display for Comp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comp::Eq => write!(f, "eq"),
            Comp::Ne => write!(f, "ne"),
            Comp::Lt => write!(f, "ult"),
            Comp::Le => write!(f, "ule"),
            Comp::Gt => write!(f, "ugt"),
            Comp::Ge => write!(f, "uge"),
            Comp::Sgt => write!(f, "sgt"),
            Comp::Sge => write!(f, "sge"),
            Comp::Slt => write!(f, "slt"),
            Comp::Sle => write!(f, "sle"),
        }
    }
}

#[derive(Debug)]
pub enum Instruction<'m> {
    Alloca(TyNVVM<'m>, u64),
    ExtractValue(TyNVVM<'m>, Val<'m>, u64),
    /// Store a value to a pointer
    Store {
        val: Val<'m>,
        ptr: Val<'m>,
        align: u64,
    },
    Load {
        ty: TyNVVM<'m>,
        ptr: Val<'m>,
        align: u64,
    },
    InBoundsGep {
        ty: TyNVVM<'m>,
        ptr: Val<'m>,
        indices: Vec<Val<'m>>,
    },

    Sub(Val<'m>, Val<'m>),
    Add(Val<'m>, Val<'m>),

    And(Val<'m>, Val<'m>),
    Or(Val<'m>, Val<'m>),
    Xor(Val<'m>, Val<'m>),

    ICmp(Comp, Val<'m>, Val<'m>),

    BitCast {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    Trunc {
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    SExt {
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    ZExt {
        val: Val<'m>,
        to: TyNVVM<'m>,
    },

    Branch(&'m BasicBlock<'m>),
    ConditionalBranch {
        cond: Val<'m>,
        true_block: &'m BasicBlock<'m>,
        false_block: &'m BasicBlock<'m>,
    },
    Unreachable,

    Retvoid,
    Ret(Val<'m>),

    Call {
        ret_ty: TyNVVM<'m>,
        fn_val: Val<'m>,
        fn_ty: TyNVVM<'m>,
        args: Vec<Val<'m>>,
    },

    MemCpy {
        dst: Val<'m>,
        dst_align: u64,
        src: Val<'m>,
        src_align: u64,
        size: Val<'m>,
        is_volatile: bool,
    }
}

#[derive(Debug)]
pub enum Const {
    I(i64),
    U(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Lit(String),
}

#[derive(Debug)]
pub enum ValueNVVM<'m> {
    Param { 
        func_name: String,
        idx: usize,
        ty: TyNVVM<'m>
    },
    Instr(Instruction<'m>),
    Constant(Const),
    Global(&'m GlobalNVVM<'m>), // a pointer to a global
    Type(TyNVVM<'m>),
    FnRef(String),
}
pub type Val<'m> = Interned<'m, ValueNVVM<'m>>;

impl<'m> ToVal<'m> for ValueNVVM<'m> {
    fn to_val(self, module: &mut ModuleNVVM<'m>) -> Val<'m> {
        module.create_val(self, None)
    }
}

impl<'m> PartialEq for ValueNVVM<'m> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
const PARAM_LABELS: [&str; 26] = ["%a", "%b", "%c", "%d", "%e", "%f", "%g", "%h", "%i", "%j", "%k", "%l", 
                      "%m", "%n", "%o", "%p", "%q", "%r", "%s", "%t", "%u", "%v", "%w", "%x", "%y", "%z"];

impl<'m> AssembleVal<'m> for Val<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>, func: &FunctionNVVM<'m>) -> String {
        match self.0 {
            ValueNVVM::Param { .. }
            | ValueNVVM::Instr(_) => {
                func.label_of_val(*self).to_string()
            }

            _ => self.0.assemble(module, func, self)
        }
    }
}

impl<'m> ValueNVVM<'m> {
    pub fn assemble(&self, module: &mut ModuleNVVM<'m>, func: &FunctionNVVM<'m>, value: &Val<'m>) -> String {
        match self {
            ValueNVVM::Param { func_name, idx, ty } => {
                func.create_val_label(*value, PARAM_LABELS[*idx].to_string());
                format!("{} {}", ty.assemble(module), PARAM_LABELS[*idx])
            }
            ValueNVVM::Instr(instr) => {
                if instr.has_ret() {
                    let label = func.assign_label_to_val(*value);
                    let instr = instr.assemble(module, func, value);
                    format!("{} = {}", label, instr)
                } else {
                    instr.assemble(module, func, value)
                }
            }
            ValueNVVM::Constant(c) => {
                match c {
                    Const::I(i) => format!("{}", i),
                    Const::U(u) => format!("{}", u),
                    Const::F32(f) => format!("{}", f),
                    Const::F64(f) => format!("{}", f),
                    Const::Bool(b) => format!("{}", b),
                    Const::Lit(s) => format!("\"{}\"", s),
                }
            }
            ValueNVVM::Type(ty) => {
                ty.assemble(module)
            }

            ValueNVVM::FnRef(name) => {
                format!("@{}", name)
            }

            ValueNVVM::Global(g) => {
                format!("@{}", g.name)
            }
        }
    }
}

impl<'m> Instruction<'m> {
    /// whether the instruction supports %x = instr
    pub fn has_ret(&self) -> bool {
        match self {
            Instruction::Alloca(..)
            | Instruction::ExtractValue(_, _, _)
            | Instruction::BitCast{ .. }
            | Instruction::Load { .. }

            | Instruction::Sub(_, _)
            | Instruction::Add(_, _)

            | Instruction::And(_, _)
            | Instruction::Or(_, _)
            | Instruction::Xor(_, _)

            | Instruction::ICmp(_, _, _)

            | Instruction::Trunc { .. }
            | Instruction::SExt { .. }
            | Instruction::ZExt { .. }
            | Instruction::InBoundsGep { .. } => true,

            Instruction::Retvoid
            | Instruction::Ret(_)
            | Instruction::Branch(_)
            | Instruction::ConditionalBranch { .. }
            | Instruction::Unreachable
            | Instruction::MemCpy { .. }
            | Instruction::Store { .. } => false,

            Instruction::Call { ret_ty, .. } => ret_ty.size() != 0,
        }
    }
}

impl<'m> Instruction<'m> {
    pub fn assemble(&self, module: &mut ModuleNVVM<'m>, func: &FunctionNVVM<'m>, val: &Val<'m>) -> String {
        match self {
            Instruction::Alloca(ty, size) => {
                format!("alloca {}, i64 {}", ty.assemble(module), size)
            }
            Instruction::ExtractValue(ty, val, idx) => {
                format!("extractvalue {} {}, {}", ty.assemble(module), val.assemble(module, func), idx)
            }
            Instruction::Store { val, ptr, align } => {
                // get the labels
                let val_ty = *module.valtypes.get(val).unwrap();
                let val_ty_str = val_ty.assemble(module);
                let ptr_ty_str = module.ty_from_type(TypeNVVM::Pointer(val_ty)).assemble(module);
                let val_label = val.assemble(module, func);
                let ptr_label = ptr.assemble(module, func);
                format!("store {} {}, {} {}, align {}", val_ty_str, val_label, ptr_ty_str, ptr_label, align)
            }
            Instruction::Load { ty, ptr, align } => {
                format!("load {} {}, align {}", ty.assemble(module), ptr.assemble(module, func), align)
            }
            Instruction::InBoundsGep { ty, ptr, indices } => {
                let ty_str = ty.assemble(module);
                let ptr_ty = *module.valtypes.get(ptr).unwrap();
                let ptr_ty_str = ptr_ty.assemble(module);
                let ptr_label = ptr.assemble(module, func);
                let mut s = format!("getelementptr inbounds {}, {} {}", ty_str, ptr_ty_str, ptr_label);
                for (i, idx) in indices.iter().enumerate() {
                    let ty = *module.valtypes.get(idx).unwrap();
                    let ty_str = ty.assemble(module);
                    let label = idx.assemble(module, func);
                    s.push_str(&format!(",{} {}", ty_str, label));
                }
                s
            }
            
            Instruction::Sub(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("sub {} {}, {}", ty_label, a.assemble(module, func), b.assemble(module, func))
            }
            Instruction::Add(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("add {} {}, {}", ty_label, a.assemble(module, func), b.assemble(module, func))
            }
            Instruction::And(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("and {} {}, {}", ty_label, a.assemble(module, func), b.assemble(module, func))
            }
            Instruction::Or(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("or {} {}, {}", ty_label, a.assemble(module, func), b.assemble(module, func))
            }
            Instruction::Xor(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("xor {} {}, {}", ty_label, a.assemble(module, func), b.assemble(module, func))
            }


            Instruction::ICmp(comp, a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("icmp {} {} {}, {}", comp, ty_label, a.assemble(module, func), b.assemble(module, func))
            }

            Instruction::BitCast { ty, val, to } => {
                let val_label = val.assemble(module, func);
                let ty_label = ty.assemble(module);
                let to_label = to.assemble(module);
                format!("bitcast {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::Trunc { val, to } => {
                let ty = *module.valtypes.get(val).unwrap();
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("trunc {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::SExt { val, to } => {
                let ty = *module.valtypes.get(val).unwrap();
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("sext {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::ZExt { val, to } => {
                let ty = *module.valtypes.get(val).unwrap();
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("zext {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::Retvoid => {
                format!("ret void")
            }
            Instruction::Ret(val) => {
                format!("ret {}", val.assemble(module, func))
            }

            Instruction::Branch(bb) => {
                format!("br label %{}", bb.name)
            }

            Instruction::ConditionalBranch { cond, true_block, false_block } => {
                format!("br i1 {}, label %{}, label %{}", cond.assemble(module, func), true_block.name, false_block.name)
            }

            Instruction::Unreachable => {
                format!("unreachable")
            }

            Instruction::Call { ret_ty, fn_val, fn_ty, args } => {
                let ret_ty_label = ret_ty.assemble(module);
                let fn_val_label = fn_val.assemble(module, func);
                let fn_ty_label = fn_ty.assemble(module);
                let mut s = format!("call {} {}(", ret_ty_label, fn_val_label);
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    // add the type of the argument as well for calls
                    let ty = *module.valtypes.get(arg).unwrap();
                    let ty_label = ty.assemble(module);
                    s.push_str(&format!("{} {}", ty_label, arg.assemble(module, func)));
                }
                s.push_str(")");
                s
            }

            Instruction::MemCpy { 
                dst, 
                dst_align, 
                src, 
                src_align, 
                size, 
                is_volatile } => {
                    // build the correct intrinsic call
                    let dst_ty = *module.valtypes.get(dst).unwrap();
                    let src_ty = *module.valtypes.get(src).unwrap();
                    let size_ty = *module.valtypes.get(size).unwrap();

                    let dst_label = dst.assemble(module, func);
                    let src_label = src.assemble(module, func);


                    let dst_ty_str = dst_ty.assemble(module);
                    let src_ty_str = src_ty.assemble(module);
                    let size_ty_str = size_ty.assemble(module);

                    let intrinsic_name = format!("llvm.memcpy.p0{}.p0{}.{}",
                        dst_ty_str, src_ty_str, size_ty_str
                    );

                    println!("Intrinsic name: {}", intrinsic_name);
                    if module.get_intrinsic(&intrinsic_name) == None {
                        // declare the intrinsic
                        let mut args = vec![];
                        let dst_ty_ptr = module.ty_from_type(TypeNVVM::Pointer(dst_ty));
                        let src_ty_ptr = module.ty_from_type(TypeNVVM::Pointer(src_ty));
                        args.push(dst_ty_ptr);
                        args.push(src_ty_ptr);
                        args.push(size_ty);

                        let zst = module.ty_from_type(TypeNVVM::Zst);
                        
                        module.declare_intrinsic(&intrinsic_name, args, zst);
                    } 
                    format!("call void @{}({} {}, {} {}, {} {}, i1 {})",
                        intrinsic_name,
                        dst_ty_str, dst_label,
                        src_ty_str, src_label,
                        size_ty_str, size.assemble(module, func),
                        if *is_volatile { "true" } else { "false" }
                    )
                }
        }
    }
}