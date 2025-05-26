use std::{cell::UnsafeCell, fmt::Display};

use crate::{
    basic_block::BasicBlock,
    function::FunctionNVVM,
    module::{Assemble, ModuleNVVM},
    ty::{self, TyNVVM, TypeNVVM},
    Global, GlobalNVVM,
};
use rustc_data_structures::intern::Interned;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs;

pub trait ToVal<'m> {
    fn to_val(self, module: &mut ModuleNVVM<'m>) -> Val<'m>;
}

pub trait AssembleVal<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>, func: &FunctionNVVM<'m>) -> String;
    fn assemble_const(&self, module: &mut ModuleNVVM<'m>) -> String;
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
    //Alloca(TyNVVM<'m>, u64),
    Alloca {
        ty: TyNVVM<'m>,
        size: u64,
        align: u64,
    },
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

    Sub {
        lhs: Val<'m>,
        rhs: Val<'m>,
        nsw: bool,
        nuw: bool,
    },
    Add {
        lhs: Val<'m>,
        rhs: Val<'m>,
        nsw: bool,
        nuw: bool,
    },

    And(Val<'m>, Val<'m>),
    Or(Val<'m>, Val<'m>),
    Xor(Val<'m>, Val<'m>),

    ICmp(Comp, Val<'m>, Val<'m>),

    URem(Val<'m>, Val<'m>),
    SRem(Val<'m>, Val<'m>),
    FRem(Val<'m>, Val<'m>),

    UDiv(Val<'m>, Val<'m>),
    SDiv(Val<'m>, Val<'m>),
    FDiv(Val<'m>, Val<'m>),

    Mul {
        lhs: Val<'m>,
        rhs: Val<'m>,
        nsw: bool,
        nuw: bool,
    },

    BitCast {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    PtrToInt {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    IntToPtr {
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

    // Shifts
    Shl {
        lhs: Val<'m>,
        rhs: Val<'m>,
        nsw: bool,
        nuw: bool,
    },

    LShr {
        lhs: Val<'m>,
        rhs: Val<'m>,
    },

    InsertValue {
        aggregate: Val<'m>,
        elt: Val<'m>,
        idx: u64,
    },

    Branch(&'m BasicBlock<'m>),
    ConditionalBranch {
        cond: Val<'m>,
        true_block: &'m BasicBlock<'m>,
        false_block: &'m BasicBlock<'m>,
    },
    Unreachable,
    Switch {
        val: Val<'m>,
        default: &'m BasicBlock<'m>,
        cases: Vec<(Val<'m>, &'m BasicBlock<'m>)>,
    },

    Phi {
        ty: TyNVVM<'m>,
        incoming: UnsafeCell<Vec<(Val<'m>, &'m BasicBlock<'m>)>>,
    },

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
    },

    LandingPad {
        ty: TyNVVM<'m>,
        num_clauses: usize,
        cleanup: bool,
    },

    Invoke {
        ty: TyNVVM<'m>,
        fn_val: Val<'m>,
        args: Vec<Val<'m>>,
        fn_attrs: Option<CodegenFnAttrs>,
        then: &'m BasicBlock<'m>,
        catch: &'m BasicBlock<'m>,
    },

    Resume(Val<'m>),

    // TEMPORARY INSTRUCTIONS (TO BE OPTIMIZED OUT)
    LifetimeStart(Val<'m>, usize),
    LifetimeEnd(Val<'m>, usize),
}

#[derive(Debug)]
pub enum Const {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Bool(bool),
    Lit(String),
    Arr(Vec<Const>),
    Undef,
}

impl Const {
    pub fn size(&self) -> usize {
        match self {
            Const::I8(_) => 8,
            Const::I16(_) => 16,
            Const::I32(_) => 32,
            Const::I64(_) => 64,
            Const::I128(_) => 128,
            Const::U8(_) => 8,
            Const::U16(_) => 16,
            Const::U32(_) => 32,
            Const::U64(_) => 64,
            Const::U128(_) => 128,
            Const::F32(_) => 32,
            Const::F64(_) => 64,
            Const::Bool(_) => 1,
            Const::Lit(s) => s.as_bytes().len(),
            Const::Undef => 0,
            Const::Arr(l) => l.len(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Const::I8(i) => Some(*i as i64),
            Const::I16(i) => Some(*i as i64),
            Const::I32(i) => Some(*i as i64),
            Const::I64(i) => Some(*i),
            Const::I128(i) => None,
            Const::U8(i) => Some(*i as i64),
            Const::U16(i) => Some(*i as i64),
            Const::U32(i) => Some(*i as i64),
            Const::U64(i) => Some(*i as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Const::U8(i) => Some(*i as u64),
            Const::U16(i) => Some(*i as u64),
            Const::U32(i) => Some(*i as u64),
            Const::U64(i) => Some(*i),
            Const::U128(i) => None,
            Const::I8(i) => Some(*i as u64),
            Const::I16(i) => Some(*i as u64),
            Const::I32(i) => Some(*i as u64),
            Const::I64(i) => Some(*i as u64),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ValueNVVM<'m> {
    Param { func_name: String, idx: usize, ty: TyNVVM<'m> },
    Instr(Instruction<'m>),
    Constant(Const),
    Global(Global<'m>), // a pointer to a global
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
const PARAM_LABELS: [&str; 26] = [
    "%a", "%b", "%c", "%d", "%e", "%f", "%g", "%h", "%i", "%j", "%k", "%l", "%m", "%n", "%o", "%p",
    "%q", "%r", "%s", "%t", "%u", "%v", "%w", "%x", "%y", "%z",
];

impl<'m> AssembleVal<'m> for Val<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>, func: &FunctionNVVM<'m>) -> String {
        match self.0 {
            ValueNVVM::Param { .. } | ValueNVVM::Instr(_) => module.label_of_val(*self, Some(func)),

            _ => self.0.assemble(module, Some(func), self),
        }
    }
    fn assemble_const(&self, module: &mut ModuleNVVM<'m>) -> String {
        match self.0 {
            ValueNVVM::Param { .. } | ValueNVVM::Instr(_) => module.label_of_val(*self, None),

            _ => self.0.assemble(module, None, self),
        }
    }
}

impl<'m> ValueNVVM<'m> {
    pub fn assemble(
        &self,
        module: &mut ModuleNVVM<'m>,
        function: Option<&FunctionNVVM<'m>>,
        value: &Val<'m>,
    ) -> String {
        match (self, function) {
            (ValueNVVM::Param { func_name, idx, ty }, Some(func)) => {
                func.create_val_label(*value, PARAM_LABELS[*idx].to_string());
                format!("{} {}", ty.assemble(module), PARAM_LABELS[*idx])
            }
            (ValueNVVM::Instr(instr), Some(func)) => {
                if instr.has_ret() {
                    let label = func.assign_label_to_val(*value);
                    let instr = instr.assemble(module, func, value);
                    format!("{} = {}", label, instr)
                } else {
                    instr.assemble(module, func, value)
                }
            }
            (ValueNVVM::Instr(instr), None) => {
                // if there is no function, we cannot assign a label
                return instr.assemble_global(module);
            }
            (ValueNVVM::Constant(c), _) => c.assemble(module),
            (ValueNVVM::Type(ty), _) => ty.assemble(module),

            (ValueNVVM::FnRef(name), _) => {
                format!("@{}", name)
            }

            (ValueNVVM::Global(g), _) => {
                format!("@{}", g.name)
            }
            (_, f) => {
                panic!("Invalid value: {:#?}, {:#?}", self, f);
            }
        }
    }
}

impl<'m> Const {
    pub fn assemble(&self, module: &mut ModuleNVVM<'m>) -> String {
        // the same as assemble for const, but without the type
        match self {
            Const::I8(i) => format!("{}", i),
            Const::I16(i) => format!("{}", i),
            Const::I32(i) => format!("{}", i),
            Const::I64(i) => format!("{}", i),
            Const::I128(i) => format!("{}", i),
            Const::U8(i) => format!("{}", i),
            Const::U16(i) => format!("{}", i),
            Const::U32(i) => format!("{}", i),
            Const::U64(i) => format!("{}", i),
            Const::U128(i) => format!("{}", i),
            Const::F32(f) => format!("{}", f),
            Const::F64(f) => format!("{}", f),
            Const::Bool(b) => format!("{}", if *b { 1 } else { 0 }),
            Const::Lit(s) => format!("\"{}\"", s),
            Const::Undef => format!("undef"),
            Const::Arr(l) => {
                let mut s = format!("[");
                for (i, c) in l.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }

                    s.push_str(&c.assemble(module));
                }
                s.push_str("]");
                s
            }
        }
    }
    pub fn assemble_for_const(&self, module: &mut ModuleNVVM<'m>) -> String {
        let ty = self.get_ty(module);
        let ty = ty.assemble(module);
        match self {
            Const::I8(i) => format!("{} {}", ty, i),
            Const::I16(i) => format!("{} {}", ty, i),
            Const::I32(i) => format!("{} {}", ty, i),
            Const::I64(i) => format!("{} {}", ty, i),
            Const::I128(i) => format!("{} {}", ty, i),
            Const::U8(i) => format!("{} {}", ty, i),
            Const::U16(i) => format!("{} {}", ty, i),
            Const::U32(i) => format!("{} {}", ty, i),
            Const::U64(i) => format!("{} {}", ty, i),
            Const::U128(i) => format!("{} {}", ty, i),
            Const::F32(f) => format!("{} {}", ty, f),
            Const::F64(f) => format!("{} {}", ty, f),
            Const::Bool(b) => format!("{} {}", ty, if *b { 1 } else { 0 }),
            Const::Lit(s) => format!("\"{}\"", s),
            Const::Undef => format!("undef"),
            Const::Arr(l) => {
                let mut s = format!("[");
                for (i, c) in l.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }

                    s.push_str(&c.assemble_for_const(module));
                }
                s.push_str("]");
                s
            }
        }
    }

    pub fn get_ty(&self, module: &mut ModuleNVVM<'m>) -> TyNVVM<'m> {
        match self {
            Const::I8(_) => module.ty_from_type(TypeNVVM::I(8)),
            Const::I16(_) => module.ty_from_type(TypeNVVM::I(16)),
            Const::I32(_) => module.ty_from_type(TypeNVVM::I(32)),
            Const::I64(_) => module.ty_from_type(TypeNVVM::I(64)),
            Const::I128(_) => module.ty_from_type(TypeNVVM::I(128)),
            Const::U8(_) => module.ty_from_type(TypeNVVM::I(8)),
            Const::U16(_) => module.ty_from_type(TypeNVVM::I(16)),
            Const::U32(_) => module.ty_from_type(TypeNVVM::I(32)),
            Const::U64(_) => module.ty_from_type(TypeNVVM::I(64)),
            Const::U128(_) => module.ty_from_type(TypeNVVM::I(128)),
            Const::F32(_) => module.ty_from_type(TypeNVVM::F32),
            Const::F64(_) => module.ty_from_type(TypeNVVM::F64),
            Const::Bool(_) => module.ty_from_type(TypeNVVM::I(1)),
            Const::Lit(_) => module.ty_from_type(TypeNVVM::I(8)),
            Const::Undef => module.ty_from_type(TypeNVVM::Zst),
            Const::Arr(l) => {
                let ty = if l.is_empty() {
                    module.ty_from_type(TypeNVVM::Zst)
                } else {
                    l[0].get_ty(module)
                };
                module.ty_from_type(TypeNVVM::Array(ty, l.len()))
            }
        }
    }
}

impl<'m> Instruction<'m> {
    /// whether the instruction supports %x = instr
    pub fn has_ret(&self) -> bool {
        match self {
            Instruction::Alloca { .. }
            | Instruction::ExtractValue(_, _, _)
            | Instruction::BitCast { .. }
            | Instruction::PtrToInt { .. }
            | Instruction::IntToPtr { .. }
            | Instruction::Load { .. }
            | Instruction::Sub { .. }
            | Instruction::Add { .. }
            | Instruction::And(_, _)
            | Instruction::Or(_, _)
            | Instruction::Xor(_, _)
            | Instruction::ICmp(_, _, _)
            | Instruction::URem(_, _)
            | Instruction::SRem(_, _)
            | Instruction::FRem(_, _)
            | Instruction::UDiv(_, _)
            | Instruction::SDiv(_, _)
            | Instruction::FDiv(_, _)
            | Instruction::Mul { .. }
            | Instruction::Shl { .. }
            | Instruction::LShr { .. }
            | Instruction::Trunc { .. }
            | Instruction::SExt { .. }
            | Instruction::ZExt { .. }
            | Instruction::LandingPad { .. }
            | Instruction::Invoke { .. }
            | Instruction::InsertValue { .. }
            | Instruction::Phi { .. }
            | Instruction::InBoundsGep { .. } => true,

            Instruction::Retvoid
            | Instruction::Ret(_)
            | Instruction::Branch(_)
            | Instruction::ConditionalBranch { .. }
            | Instruction::Unreachable
            | Instruction::MemCpy { .. }
            | Instruction::Switch { .. }
            | Instruction::Resume(_)
            | Instruction::LifetimeStart(_, _)
            | Instruction::LifetimeEnd(_, _)
            | Instruction::Store { .. } => false,

            Instruction::Call { ret_ty, .. } => !ret_ty.is_zst(),
        }
    }
}

impl<'m> Instruction<'m> {
    pub fn assemble(
        &self,
        module: &mut ModuleNVVM<'m>,
        func: &FunctionNVVM<'m>,
        val: &Val<'m>,
    ) -> String {
        match self {
            Instruction::Alloca { ty, size, align } => {
                format!("alloca {}, i64 {}, align {}", ty.assemble(module), size, align)
            }
            Instruction::ExtractValue(ty, val, idx) => {
                format!(
                    "extractvalue {} {}, {}",
                    ty.assemble(module),
                    val.assemble(module, func),
                    idx
                )
            }
            Instruction::Store { val, ptr, align } => {
                // get the labels
                let val_ty = *module.valtypes.get(val).unwrap();
                let val_ty_str = val_ty.assemble(module);
                let ptr_ty_str = module.ty_from_type(TypeNVVM::Pointer(val_ty)).assemble(module);
                let val_label = val.assemble(module, func);
                let ptr_label = ptr.assemble(module, func);
                format!(
                    "store {} {}, {} {}, align {}",
                    val_ty_str, val_label, ptr_ty_str, ptr_label, align
                )
            }
            Instruction::Load { ty, ptr, align } => {
                let ptr_ty = *module.valtypes.get(ptr).unwrap();
                let ptr_ty_str = ptr_ty.assemble(module);
                format!(
                    "load {}, {} {}, align {}",
                    ty.assemble(module),
                    ptr_ty_str,
                    ptr.assemble(module, func),
                    align
                )
            }
            Instruction::InBoundsGep { ty, ptr, indices } => {
                let ty_str = ty.assemble(module);
                let ptr_ty = *module.valtypes.get(ptr).unwrap();
                let ptr_ty_str = ptr_ty.assemble(module);
                let ptr_label = ptr.assemble(module, func);
                let mut s =
                    format!("getelementptr inbounds {}, {} {}", ty_str, ptr_ty_str, ptr_label);
                for (i, idx) in indices.iter().enumerate() {
                    let ty = *module.valtypes.get(idx).unwrap();
                    let ty_str = ty.assemble(module);
                    let label = idx.assemble(module, func);
                    s.push_str(&format!(",{} {}", ty_str, label));
                }
                s
            }

            Instruction::Sub { lhs, rhs, nsw, nuw } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "sub {} {}, {}",
                    ty_label,
                    lhs.assemble(module, func),
                    rhs.assemble(module, func)
                )
            }
            Instruction::Add { lhs, rhs, nsw, nuw } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                let opts = match (nsw, nuw) {
                    (true, true) => "nsw nuw ",
                    (true, false) => "nsw ",
                    (false, true) => "nuw ",
                    (false, false) => "",
                };
                format!(
                    "add {}{} {}, {}",
                    opts,
                    ty_label,
                    lhs.assemble(module, func),
                    rhs.assemble(module, func)
                )
            }
            Instruction::And(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "and {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }
            Instruction::Or(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "or {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }
            Instruction::Xor(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "xor {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::ICmp(comp, a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "icmp {} {} {}, {}",
                    comp,
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::URem(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "urem {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::SRem(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "srem {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::FRem(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "frem {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::UDiv(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "udiv {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::SDiv(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "sdiv {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::FDiv(a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "fdiv {} {}, {}",
                    ty_label,
                    a.assemble(module, func),
                    b.assemble(module, func)
                )
            }

            Instruction::Mul { lhs, rhs, nsw, nuw } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                let opts = match (nsw, nuw) {
                    (true, true) => "nsw nuw ",
                    (true, false) => "nsw ",
                    (false, true) => "nuw ",
                    (false, false) => "",
                };
                format!(
                    "mul {}{} {}, {}",
                    opts,
                    ty_label,
                    lhs.assemble(module, func),
                    rhs.assemble(module, func)
                )
            }

            Instruction::Shl { lhs, rhs, nsw, nuw } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                let opts = match (nsw, nuw) {
                    (true, true) => "nsw nuw ",
                    (true, false) => "nsw ",
                    (false, true) => "nuw ",
                    (false, false) => "",
                };
                format!(
                    "shl {}{} {}, {}",
                    opts,
                    ty_label,
                    lhs.assemble(module, func),
                    rhs.assemble(module, func)
                )
            }

            Instruction::LShr { lhs, rhs } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "lshr {} {}, {}",
                    ty_label,
                    lhs.assemble(module, func),
                    rhs.assemble(module, func)
                )
            }

            Instruction::BitCast { ty, val, to } => {
                let val_label = val.assemble(module, func);
                let ty_label = ty.assemble(module);
                let to_label = to.assemble(module);
                format!("bitcast {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::PtrToInt { ty, val, to } => {
                let val_label = val.assemble(module, func);
                let ty_label = ty.assemble(module);
                let to_label = to.assemble(module);
                format!("ptrtoint {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::IntToPtr { ty, val, to } => {
                let val_label = val.assemble(module, func);
                let ty_label = ty.assemble(module);
                let to_label = to.assemble(module);
                format!("inttoptr {} {} to {}", ty_label, val_label, to_label)
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

            Instruction::InsertValue { aggregate, elt, idx } => {
                let ty = *module.valtypes.get(aggregate).unwrap();
                let ty_label = ty.assemble(module);
                let aggregate_label = aggregate.assemble(module, func);
                let elt_ty = *module.valtypes.get(elt).unwrap();
                let elt_label = elt.assemble(module, func);
                format!(
                    "insertvalue {} {}, {} {}, {}",
                    ty_label,
                    aggregate_label,
                    elt_ty.assemble(module),
                    elt_label,
                    idx
                )
            }

            Instruction::Retvoid => {
                format!("ret void")
            }
            Instruction::Ret(val) => {
                let ty = *module.valtypes.get(val).unwrap();
                let ty_label = ty.assemble(module);
                format!("ret {} {}", ty_label, val.assemble(module, func))
            }

            Instruction::Branch(bb) => {
                format!("br label %{}", bb.name)
            }

            Instruction::ConditionalBranch { cond, true_block, false_block } => {
                format!(
                    "br i1 {}, label %{}, label %{}",
                    cond.assemble(module, func),
                    true_block.name,
                    false_block.name
                )
            }

            Instruction::Unreachable => {
                format!("unreachable")
            }

            Instruction::Switch { val, default, cases } => {
                let int_ty = *module.valtypes.get(val).unwrap();
                let int_ty = int_ty.assemble(module);
                let val_label = val.assemble(module, func);
                let mut s = format!("switch {} {}, label %{} [", int_ty, val_label, default.name);
                for (i, (case, bb)) in cases.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!("{} {} %{}", int_ty, case.assemble(module, func), bb.name));
                }
                s.push_str("]");
                s
            }

            Instruction::Phi { ty, incoming } => {
                let ty_label = ty.assemble(module);
                let mut s = format!("phi {} ", ty_label);
                let incoming = unsafe { &*incoming.get() };
                for (i, (val, bb)) in incoming.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!("[{}, %{}]", val.assemble(module, func), bb.name));
                }
                s
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

            Instruction::MemCpy { dst, dst_align, src, src_align, size, is_volatile } => {
                // build the correct intrinsic call
                let dst_ty = *module.valtypes.get(dst).unwrap();
                let src_ty = *module.valtypes.get(src).unwrap();
                let size_ty = *module.valtypes.get(size).unwrap();

                let dst_label = dst.assemble(module, func);
                let src_label = src.assemble(module, func);

                let dst_ty_str = dst_ty.assemble(module);
                let src_ty_str = src_ty.assemble(module);
                let size_ty_str = size_ty.assemble(module);

                let intrinsic_name =
                    format!("llvm.memcpy.p0{}.p0{}.{}", dst_ty_str, src_ty_str, size_ty_str);

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
                format!(
                    "call void @{}({} {}, {} {}, {} {}, i1 {})",
                    intrinsic_name,
                    dst_ty_str,
                    dst_label,
                    src_ty_str,
                    src_label,
                    size_ty_str,
                    size.assemble(module, func),
                    if *is_volatile { "true" } else { "false" }
                )
            }

            Instruction::LandingPad { ty, num_clauses, cleanup } => {
                let ty_label = ty.assemble(module);
                format!(
                    "landingpad {} cleanup {}",
                    ty_label,
                    if *cleanup { "cleanup" } else { "catch" }
                )
            }

            Instruction::Invoke { ty, fn_val, args, fn_attrs, then, catch } => {
                let ty_label = ty.assemble(module);
                let fn_val_label = fn_val.assemble(module, func);
                let mut s = format!("invoke {} {}(", ty_label, fn_val_label);
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    let ty = *module.valtypes.get(arg).unwrap();
                    let ty_label = ty.assemble(module);
                    s.push_str(&format!("{} {}", ty_label, arg.assemble(module, func)));
                }
                s.push_str(")");
                s.push_str(&format!(" to label %{} unwind label %{}", then.name, catch.name));
                s
            }

            Instruction::Resume(val) => {
                panic!("Resume instruction not supported in nvvm")
            }

            // TEMPORARY INSTRUCTIONS (TO BE OPTIMIZED OUT)
            Instruction::LifetimeStart(_, _) => {
                panic!("Lifetime is a temporary instruction")
            }
            Instruction::LifetimeEnd(_, _) => {
                panic!("Lifetime is a temporary instruction")
            }
        }
    }

    pub fn assemble_global(&self, module: &mut ModuleNVVM<'m>) -> String {
        match self {
            Instruction::InBoundsGep { ty, ptr, indices } => {
                let ty_str = ty.assemble(module);
                let ptr_ty = *module.valtypes.get(ptr).unwrap();
                let ptr_ty_str = ptr_ty.assemble(module);
                let ptr_label = ptr.assemble_const(module);
                let mut s =
                    format!("getelementptr inbounds {}, {} {}", ty_str, ptr_ty_str, ptr_label);
                for (i, idx) in indices.iter().enumerate() {
                    let ty = *module.valtypes.get(idx).unwrap();
                    let ty_str = ty.assemble(module);
                    let label = idx.assemble_const(module);
                    s.push_str(&format!(",{} {}", ty_str, label));
                }
                s
            }
            _ => panic!("Invalid instruction for global: {:?}", self),
        }
    }
}
