use std::fmt::Display;

use rustc_data_structures::intern::Interned;
use crate::{basic_block::BasicBlock, function::FunctionNVVM, module::{Assemble, ModuleNVVM}, ty::{TyNVVM, TypeNVVM}, GlobalNVVM};

pub trait ToVal<'m> {
    fn to_val(self, module: &mut ModuleNVVM<'m>) -> Val<'m>;
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

impl<'m> Assemble<'m> for Val<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>) -> String {
        match self.0 {
            ValueNVVM::Param { .. }
            | ValueNVVM::Instr(_) => {
                module.label_of_val(*self).to_string()
            }

            _ => self.0.assemble(module, self)
        }
    }
}

impl<'m> ValueNVVM<'m> {
    pub fn assemble(&self, module: &mut ModuleNVVM<'m>, value: &Val<'m>) -> String {
        match self {
            ValueNVVM::Param { func_name, idx, ty } => {
                module.create_val_label(*value, PARAM_LABELS[*idx].to_string());
                format!("{} {}", ty.assemble(module), PARAM_LABELS[*idx])
            }
            ValueNVVM::Instr(instr) => {
                if instr.has_ret() {
                    let instr = instr.assemble(module, value);
                    let label = module.label_of_val(*value);
                    format!("{} = {}", label, instr)
                } else {
                    instr.assemble(module, value)
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
            | Instruction::Store { .. } => false,

            Instruction::Call { ret_ty, .. } => ret_ty.size() != 0,
        }
    }
}

impl<'m> Instruction<'m> {
    pub fn assemble(&self, module: &mut ModuleNVVM<'m>, val: &Val<'m>) -> String {
        match self {
            Instruction::Alloca(ty, size) => {
                format!("alloca {}, i64 {}", ty.assemble(module), size)
            }
            Instruction::ExtractValue(ty, val, idx) => {
                format!("extractvalue {} {}, {}", ty.assemble(module), val.assemble(module), idx)
            }
            Instruction::Store { val, ptr, align } => {
                // get the labels
                let val_ty = *module.valtypes.get(val).unwrap();
                let val_ty_str = val_ty.assemble(module);
                let ptr_ty_str = module.ty_from_type(TypeNVVM::Pointer(val_ty)).assemble(module);
                let val_label = val.assemble(module);
                let ptr_label = ptr.assemble(module);
                format!("store {} {}, {} {}, align {}", val_ty_str, val_label, ptr_ty_str, ptr_label, align)
            }
            Instruction::Load { ty, ptr, align } => {
                format!("load {} {}, align {}", ty.assemble(module), ptr.assemble(module), align)
            }
            Instruction::InBoundsGep { ty, ptr, indices } => {
                let ty_str = ty.assemble(module);
                let ptr_ty = *module.valtypes.get(ptr).unwrap();
                let ptr_ty_str = ptr_ty.assemble(module);
                let ptr_label = ptr.assemble(module);
                let mut s = format!("getelementptr inbounds {}, {} {}", ty_str, ptr_ty_str, ptr_label);
                for (i, idx) in indices.iter().enumerate() {
                    let ty = *module.valtypes.get(idx).unwrap();
                    let ty_str = ty.assemble(module);
                    let label = idx.assemble(module);
                    s.push_str(&format!(",{} {}", ty_str, label));
                }
                s
            }
            
            Instruction::Sub(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("sub {} {}, {}", ty_label, a.assemble(module), b.assemble(module))
            }
            Instruction::Add(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("add {} {}, {}", ty_label, a.assemble(module), b.assemble(module))
            }
            Instruction::And(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("and {} {}, {}", ty_label, a.assemble(module), b.assemble(module))
            }
            Instruction::Or(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("or {} {}, {}", ty_label, a.assemble(module), b.assemble(module))
            }
            Instruction::Xor(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("xor {} {}, {}", ty_label, a.assemble(module), b.assemble(module))
            }


            Instruction::ICmp(comp, a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!("icmp {} {} {}, {}", comp, ty_label, a.assemble(module), b.assemble(module))
            }

            Instruction::BitCast { ty, val, to } => {
                let val_label = val.assemble(module);
                let ty_label = ty.assemble(module);
                let to_label = to.assemble(module);
                format!("bitcast {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::Trunc { val, to } => {
                let ty = *module.valtypes.get(val).unwrap();
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module);
                let to_label = to.assemble(module);
                format!("trunc {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::SExt { val, to } => {
                let ty = *module.valtypes.get(val).unwrap();
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module);
                let to_label = to.assemble(module);
                format!("sext {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::ZExt { val, to } => {
                let ty = *module.valtypes.get(val).unwrap();
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module);
                let to_label = to.assemble(module);
                format!("zext {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::Retvoid => {
                format!("ret void")
            }
            Instruction::Ret(val) => {
                format!("ret {}", val.assemble(module))
            }

            Instruction::Branch(bb) => {
                format!("br label %{}", bb.name)
            }

            Instruction::ConditionalBranch { cond, true_block, false_block } => {
                format!("br i1 {}, label %{}, label %{}", cond.assemble(module), true_block.name, false_block.name)
            }

            Instruction::Unreachable => {
                format!("unreachable")
            }

            Instruction::Call { ret_ty, fn_val, fn_ty, args } => {
                let ret_ty_label = ret_ty.assemble(module);
                let fn_val_label = fn_val.assemble(module);
                let fn_ty_label = fn_ty.assemble(module);
                let mut s = format!("call {} {}(", ret_ty_label, fn_val_label);
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    // add the type of the argument as well for calls
                    let ty = *module.valtypes.get(arg).unwrap();
                    let ty_label = ty.assemble(module);
                    s.push_str(&format!("{} {}", ty_label, arg.assemble(module)));
                }
                s.push_str(")");
                s
            }
        }
    }
}