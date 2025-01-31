use std::{fmt::Display, ptr};

use rustc_ast::Ty;
use rustc_data_structures::intern::Interned;
use rustc_hir::def_id::DefId;
use rustc_mir_build::build;

use crate::module::{Assemble, ModuleNVVM};

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
    Union(Vec<TyNVVM<'m>>),
    Fn(Vec<TyNVVM<'m>>, TyNVVM<'m>),
    AdtDefForwardDecl(DefId, String),
}

/// Interned type for NVVM.
pub type TyNVVM<'m> = Interned<'m, TypeNVVM<'m>>;



impl<'m> PartialEq for TypeNVVM<'m> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeNVVM::Zst, TypeNVVM::Zst) => true,
            (TypeNVVM::I(a), TypeNVVM::I(b)) => a == b,
            (TypeNVVM::F32, TypeNVVM::F32) => true,
            (TypeNVVM::F64, TypeNVVM::F64) => true,
            (TypeNVVM::Pointer(a), TypeNVVM::Pointer(b)) => a == b,
            (TypeNVVM::Array(a, s1), TypeNVVM::Array(b, s2)) => a == b && s1 == s2,
            (TypeNVVM::Struct(a), TypeNVVM::Struct(b)) => a == b,
            (TypeNVVM::Union(a), TypeNVVM::Union(b)) => a == b,
            (TypeNVVM::Fn(a1, b1), TypeNVVM::Fn(a2, b2)) => a1 == a2 && b1 == b2,
            _ => false,
        }
    }
}

// assemble on ty yields the label of the type
impl<'m> Assemble<'m> for TyNVVM<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>) -> String {
        match module.get_label_of_ty(*self) {
            Some(label) => label.clone(),
            None => {
                // return the raw type
                format!("{}", self.0)
            }
        }
    }
}

impl<'m> TypeNVVM<'m> {
    pub fn size(&self) -> usize {
        match self {
            TypeNVVM::Zst => 0,
            TypeNVVM::I(size) => *size,
            TypeNVVM::F32 => 4,
            TypeNVVM::F64 => 8,
            TypeNVVM::Pointer(_) => 8,
            TypeNVVM::Array(ty, size) => ty.size() * size,
            TypeNVVM::Struct(fields) => fields.iter().map(|f| f.size()).sum(),
            TypeNVVM::Fn(_, _) => 8, // function pointer size
            TypeNVVM::Union(fields) => fields.iter().map(|f| f.size()).max().unwrap_or(0),
            TypeNVVM::AdtDefForwardDecl(_, _) => panic!("AdtDefForwardDecl should not be used for size calculation"),
        }
    }
    
}

// display for the initial declaration and simple types
impl Display for TypeNVVM<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeNVVM::Zst => write!(f, "void"),
            TypeNVVM::I(size) => write!(f, "i{}", size),
            TypeNVVM::F32 => write!(f, "float"),
            TypeNVVM::F64 => write!(f, "double"),
            TypeNVVM::Pointer(ty) => write!(f, "{}*", ty.0),
            TypeNVVM::Array(ty, size) => write!(f, "[{} x {}]", size, ty.0),
            TypeNVVM::Struct(fields) => {
                write!(f, "{{")?;
                for (i, field) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", field.0)?;
                }
                write!(f, "}}")
            }

            TypeNVVM::Fn(args, ret) => {
                write!(f, "{}(", ret.0)?;
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg.0)?;
                }
                write!(f, ")")
            }

            TypeNVVM::Union(fields) => {
                // when generating a union, pick the type with the greatest width,
                // and simply create an i8 array of that width
                let mut max_size = 0;
                for field in fields {
                    let size = field.size();
                    if size > max_size {
                        max_size = size;
                    }
                }
                write!(f, "[{} x i8]", max_size)
            }

            TypeNVVM::AdtDefForwardDecl(did, name) => {
                write!(f, "%{}", name)
            }
        }
    }
}



/// A type for inferring the final type of a value. This needs to be done 
/// because the type of a value is not given to us...
pub struct TypeHints<'m> {
    size: usize,
    /// the layout of the type in memory
    layout: Vec<(usize, usize, TyNVVM<'m>)>,
    structures: Vec<(usize, usize)>,
}

impl<'m> TypeHints<'m> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            layout: Vec::new(),
            structures: Vec::new(),
        }
    }

    pub fn layout_hint(&mut self, offset: usize, size: usize, ty: TyNVVM<'m>) {
        self.layout.push((offset, size, ty));
    }

    pub fn struct_hint(&mut self, offset: usize, size: usize) {
        self.structures.push((offset, size));
    }

    pub fn build(self, module: &mut ModuleNVVM<'m>) -> TyNVVM<'m> {
        // if the size is 0, it is a ZST
        if self.size == 0 {
            return module.ty_from_type(TypeNVVM::Zst);
        }

        // if there is only one field, it is a simple type
        if self.layout.len() == 1 {
            return self.layout[0].2;
        }

        // order the layout by offset
        let mut layout = self.layout.clone();
        layout.sort_by_key(|(offset, _, _)| *offset);

        // order the structures by offset, and then by size (largest first)
        let mut structures = self.structures.clone();
        structures.sort_by_key(|(offset, size)| (*offset, *size));

        // if there are multiple fields, it is a struct
        // we should build with the struct hints
        let mut scount = 0;
        let mut lcount = 0;
        let mut p = 0;
        self.build_struct(&layout, &structures, module, p, scount, lcount)
    }

    fn build_struct(&self, 
        layout: &[(usize, usize, TyNVVM<'m>)], 
        structures: &[(usize, usize)],
        module: &mut ModuleNVVM<'m>,
        p: usize,
        struct_idx: usize,
        layout_idx: usize,
    ) -> TyNVVM<'m> {
        let (offset, size) = structures[struct_idx];
        let mut cp = p;
        let mut sp = struct_idx;
        let mut lp = layout_idx;
        let mut types = Vec::new();
        // if there is another structure that start at p, build that struct first
        while cp < offset + size {
            if sp + 1 < structures.len() && structures[sp + 1].0 == p {
                let s = self.build_struct(layout, structures, module, p, sp + 1, layout_idx);
                cp += structures[struct_idx + 1].1;
                sp += 1;
                types.push(s);
            } else if lp < layout.len() && layout[lp].0 == p {
                let ty = layout[lp].2;
                cp += layout[lp].1;
                lp += 1;
                types.push(ty);
            } else {
                // padding, find the next field
                let mut next = usize::MAX;
                if sp + 1 < structures.len() {
                    next = structures[sp + 1].0;
                }
                if lp < layout.len() {
                    next = next.min(layout[lp].0);
                }

                let padding = next - cp;
                let i8t = module.ty_from_type(TypeNVVM::I(8));
                types.push(module.ty_from_type(TypeNVVM::Array(i8t, padding)));
                cp += padding;
            }
        }

        todo!()
    }
}