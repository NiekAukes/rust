use std::ptr;

use rustc_data_structures::intern::Interned;

/// Type tree for NVVM, it is implemented as a tree of types.
#[derive(Debug)]
pub enum TypeNVVM<'m> {
    Zst,
    I(usize),
    F32,
    F64,
    Pointer(TyNVVM<'m>),
    Array(TyNVVM<'m>, usize),
    Struct(Vec<TyNVVM<'m>>),
}

/// Interned type for NVVM.
pub type TyNVVM<'m> = Interned<'m, TypeNVVM<'m>>;



impl<'m> PartialEq for TypeNVVM<'m> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}