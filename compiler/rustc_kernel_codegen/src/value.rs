use std::{cell::UnsafeCell, fmt::Display};

use crate::{
    basic_block::BasicBlock,
    function::FunctionNVVM,
    global::{ConstExpr, GlobalNVVM},
    module::{Assemble, ModuleNVVM},
    ty::{self, TyNVVM, TypeNVVM},
};
use rustc_data_structures::intern::Interned;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs;

pub trait ToVal<'m> {
    fn to_val(self, module: &mut ModuleNVVM<'m>) -> Val<'m>;
}

pub trait AssembleVal<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>, func: &FunctionNVVM<'m>) -> String;
}

#[derive(Debug)]
pub enum IComp {
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
/*

   false: no comparison, always returns false
   oeq: ordered and equal
   ogt: ordered and greater than
   oge: ordered and greater than or equal
   olt: ordered and less than
   ole: ordered and less than or equal
   one: ordered and not equal
   ord: ordered (no nans)
   ueq: unordered or equal
   ugt: unordered or greater than
   uge: unordered or greater than or equal
   ult: unordered or less than
   ule: unordered or less than or equal
   une: unordered or not equal
   uno: unordered (either nans)
   true: no comparison, always returns true

*/

#[derive(Debug)]
pub enum FComp {
    False,
    Oeq,
    Ogt,
    Oge,
    Olt,
    Ole,
    One,
    Ord,
    Ueq,
    Ugt,
    Uge,
    Ult,
    Ule,
    Une,
    Uno,
    True,
}

impl From<rustc_codegen_ssa::common::IntPredicate> for IComp {
    fn from(pred: rustc_codegen_ssa::common::IntPredicate) -> Self {
        match pred {
            rustc_codegen_ssa::common::IntPredicate::IntEQ => IComp::Eq,
            rustc_codegen_ssa::common::IntPredicate::IntNE => IComp::Ne,
            rustc_codegen_ssa::common::IntPredicate::IntULT => IComp::Lt,
            rustc_codegen_ssa::common::IntPredicate::IntULE => IComp::Le,
            rustc_codegen_ssa::common::IntPredicate::IntUGT => IComp::Gt,
            rustc_codegen_ssa::common::IntPredicate::IntUGE => IComp::Ge,
            rustc_codegen_ssa::common::IntPredicate::IntSGT => IComp::Sgt,
            rustc_codegen_ssa::common::IntPredicate::IntSGE => IComp::Sge,
            rustc_codegen_ssa::common::IntPredicate::IntSLT => IComp::Slt,
            rustc_codegen_ssa::common::IntPredicate::IntSLE => IComp::Sle,
        }
    }
}

impl From<rustc_codegen_ssa::common::RealPredicate> for FComp {
    fn from(pred: rustc_codegen_ssa::common::RealPredicate) -> Self {
        match pred {
            rustc_codegen_ssa::common::RealPredicate::RealPredicateFalse => FComp::False,
            rustc_codegen_ssa::common::RealPredicate::RealOEQ => FComp::Oeq,
            rustc_codegen_ssa::common::RealPredicate::RealOGT => FComp::Ogt,
            rustc_codegen_ssa::common::RealPredicate::RealOGE => FComp::Oge,
            rustc_codegen_ssa::common::RealPredicate::RealOLT => FComp::Olt,
            rustc_codegen_ssa::common::RealPredicate::RealOLE => FComp::Ole,
            rustc_codegen_ssa::common::RealPredicate::RealONE => FComp::One,
            rustc_codegen_ssa::common::RealPredicate::RealORD => FComp::Ord,
            rustc_codegen_ssa::common::RealPredicate::RealUNO => FComp::Uno,
            rustc_codegen_ssa::common::RealPredicate::RealUEQ => FComp::Ueq,
            rustc_codegen_ssa::common::RealPredicate::RealUGT => FComp::Ugt,
            rustc_codegen_ssa::common::RealPredicate::RealUGE => FComp::Uge,
            rustc_codegen_ssa::common::RealPredicate::RealULT => FComp::Ult,
            rustc_codegen_ssa::common::RealPredicate::RealULE => FComp::Ule,
            rustc_codegen_ssa::common::RealPredicate::RealUNE => FComp::Une,
            rustc_codegen_ssa::common::RealPredicate::RealPredicateTrue => FComp::True,
        }
    }
}

impl Display for IComp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IComp::Eq => write!(f, "eq"),
            IComp::Ne => write!(f, "ne"),
            IComp::Lt => write!(f, "ult"),
            IComp::Le => write!(f, "ule"),
            IComp::Gt => write!(f, "ugt"),
            IComp::Ge => write!(f, "uge"),
            IComp::Sgt => write!(f, "sgt"),
            IComp::Sge => write!(f, "sge"),
            IComp::Slt => write!(f, "slt"),
            IComp::Sle => write!(f, "sle"),
        }
    }
}

impl Display for FComp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FComp::False => write!(f, "false"),
            FComp::Oeq => write!(f, "oeq"),
            FComp::Ogt => write!(f, "ogt"),
            FComp::Oge => write!(f, "oge"),
            FComp::Olt => write!(f, "olt"),
            FComp::Ole => write!(f, "ole"),
            FComp::One => write!(f, "one"),
            FComp::Ord => write!(f, "ord"),
            FComp::Ueq => write!(f, "ueq"),
            FComp::Ugt => write!(f, "ugt"),
            FComp::Uge => write!(f, "uge"),
            FComp::Ult => write!(f, "ult"),
            FComp::Ule => write!(f, "ule"),
            FComp::Une => write!(f, "une"),
            FComp::Uno => write!(f, "uno"),
            FComp::True => write!(f, "true"),
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
    FSub {
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
    FAdd {
        lhs: Val<'m>,
        rhs: Val<'m>,
        nsw: bool,
        nuw: bool,
    },

    And(Val<'m>, Val<'m>),
    Or(Val<'m>, Val<'m>),
    Xor(Val<'m>, Val<'m>),

    ICmp(IComp, Val<'m>, Val<'m>),
    FCmp(FComp, Val<'m>, Val<'m>),

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

    FMul {
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
    Select {
        cond: Val<'m>,
        then_val: Val<'m>,
        else_val: Val<'m>,
    },

    // float conversions
    FPToUI {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    FPToSI {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    UIToFP {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    SIToFP {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    FPTrunc {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },
    FExt {
        ty: TyNVVM<'m>,
        val: Val<'m>,
        to: TyNVVM<'m>,
    },

    // TEMPORARY INSTRUCTIONS (TO BE OPTIMIZED OUT)
    LifetimeStart(Val<'m>, usize),
    LifetimeEnd(Val<'m>, usize),
}

#[derive(Debug, Clone)]
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
    Struct(Vec<Const>),
    Undef,
    NullPtr,
    ZeroInitializer,
    FnRef(String),
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
            Const::Struct(l) => l.len(),
            Const::FnRef(_) => 8, // function references have pointer size
            Const::NullPtr => 8,  // null pointer has pointer size
            Const::ZeroInitializer => 0, // zero initializer has no size
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
    ConstExpr(ConstExpr<'m>), // a constant expression, e.g. GEP
    Global(GlobalNVVM<'m>),   // a pointer to a global
    Type(TyNVVM<'m>),
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
            ValueNVVM::Param { .. } | ValueNVVM::Instr(_) => {
                func.label_of_val(*self).expect("Value should have a label")
            }

            _ => self.0.assemble(module, func, self),
        }
    }
}

impl<'m> ValueNVVM<'m> {
    pub fn assemble(
        &self,
        module: &mut ModuleNVVM<'m>,
        func: &FunctionNVVM<'m>,
        value: &Val<'m>,
    ) -> String {
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
            ValueNVVM::Constant(c) => c.assemble(module),
            ValueNVVM::Type(ty) => ty.assemble(module),

            ValueNVVM::Global(g) => {
                format!("@{}", g.name)
            }

            _ => {
                panic!("Invalid value: {:#?}", self);
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
            Const::Struct(l) => {
                let mut s = format!("{{");
                for (i, c) in l.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }

                    s.push_str(&c.assemble(module));
                }
                s.push_str("}");
                s
            }

            Const::FnRef(name) => format!("@{}", name),
            Const::NullPtr => format!("null"),
            Const::ZeroInitializer => format!("zeroinitializer"),
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
                format!("{} {}", ty, s)
            }
            Const::Struct(l) => {
                let mut s = format!("{{");
                for (i, c) in l.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }

                    s.push_str(&c.assemble_for_const(module));
                }
                s.push_str("}");
                format!("{} {}", ty, s)
            }
            Const::FnRef(name) => {
                format!("{} @{}", ty, name)
            }
            Const::NullPtr => format!("{} null", ty),
            Const::ZeroInitializer => format!("{} zeroinitializer", ty),
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
            Const::Struct(l) => {
                // struc type
                let mut tys = Vec::with_capacity(l.len());
                for c in l {
                    tys.push(c.get_ty(module));
                }
                module.ty_from_type(TypeNVVM::Struct(tys))
            }

            Const::FnRef(name) => {
                // get the function from the module
                let func = *module.functions.get(name).expect("Function not found");
                let ty = func.ty;
                let ptr_ty = module.ty_from_type(TypeNVVM::Pointer(ty));
                ptr_ty
            }

            Const::NullPtr | Const::ZeroInitializer => {
                panic!(
                    "get_ty called on {:?}, which has no intrinsic type. The type must be supplied by context.",
                    self
                )
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
            | Instruction::FSub { .. }
            | Instruction::Add { .. }
            | Instruction::FAdd { .. }
            | Instruction::And(_, _)
            | Instruction::Or(_, _)
            | Instruction::Xor(_, _)
            | Instruction::ICmp(_, _, _)
            | Instruction::FCmp(_, _, _)
            | Instruction::URem(_, _)
            | Instruction::SRem(_, _)
            | Instruction::FRem(_, _)
            | Instruction::UDiv(_, _)
            | Instruction::SDiv(_, _)
            | Instruction::FDiv(_, _)
            | Instruction::Mul { .. }
            | Instruction::FMul { .. }
            | Instruction::Shl { .. }
            | Instruction::LShr { .. }
            | Instruction::Trunc { .. }
            | Instruction::SExt { .. }
            | Instruction::ZExt { .. }
            | Instruction::FExt { .. }
            | Instruction::FPToUI { .. }
            | Instruction::FPToSI { .. }
            | Instruction::UIToFP { .. }
            | Instruction::SIToFP { .. }
            | Instruction::FPTrunc { .. }
            | Instruction::BitCast { .. }
            | Instruction::LandingPad { .. }
            | Instruction::Invoke { .. }
            | Instruction::InsertValue { .. }
            | Instruction::Phi { .. }
            | Instruction::Select { .. }
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
            Instruction::FSub { lhs, rhs, nsw, nuw } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "fsub {} {}, {}",
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
            Instruction::FAdd { lhs, rhs, nsw, nuw } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                let opts = match (nsw, nuw) {
                    (true, true) => "nsw nuw ",
                    (true, false) => "nsw ",
                    (false, true) => "nuw ",
                    (false, false) => "",
                };
                format!(
                    "fadd {}{} {}, {}",
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

            Instruction::FCmp(comp, a, b) => {
                let ty = *module.valtypes.get(a).unwrap();
                let ty_label = ty.assemble(module);
                format!(
                    "fcmp {} {} {}, {}",
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

            Instruction::FMul { lhs, rhs, nsw, nuw } => {
                let ty = *module.valtypes.get(lhs).unwrap();
                let ty_label = ty.assemble(module);
                let opts = match (nsw, nuw) {
                    (true, true) => "nsw nuw ",
                    (true, false) => "nsw ",
                    (false, true) => "nuw ",
                    (false, false) => "",
                };
                format!(
                    "fmul {}{} {}, {}",
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

            Instruction::Select { cond, then_val, else_val } => {
                let cond_ty = *module.valtypes.get(cond).unwrap();
                let cond_ty_str = cond_ty.assemble(module);
                let cond_label = cond.assemble(module, func);

                let val_ty = *module.valtypes.get(then_val).unwrap();
                let val_ty_str = val_ty.assemble(module);

                let then_label = then_val.assemble(module, func);
                let else_label = else_val.assemble(module, func);

                format!(
                    "select {} {}, {} {}, {} {}",
                    cond_ty_str, cond_label, val_ty_str, then_label, val_ty_str, else_label
                )
            }

            Instruction::FPToUI { val, to, ty } => {
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("fptoui {} {} to {}", ty_label, val_label, to_label)
            }
            Instruction::FPToSI { val, to, ty } => {
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("fptosi {} {} to {}", ty_label, val_label, to_label)
            }
            Instruction::UIToFP { val, to, ty } => {
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("uitofp {} {} to {}", ty_label, val_label, to_label)
            }
            Instruction::SIToFP { val, to, ty } => {
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("sitoft {} {} to {}", ty_label, val_label, to_label)
            }

            Instruction::FPTrunc { val, to, ty } => {
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("fptrunc {} {} to {}", ty_label, val_label, to_label)
            }
            Instruction::FExt { val, to, ty } => {
                let ty_label = ty.assemble(module);
                let val_label = val.assemble(module, func);
                let to_label = to.assemble(module);
                format!("fpext {} {} to {}", ty_label, val_label, to_label)
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
}
