use std::iter::Map;

use rustc_data_structures::fx::FxHashMap;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::Ty;

use crate::{function::FunctionNVVM, GlobalNVVM, ty::{TyNVVM, TypeNVVM}, Arena};
pub struct ModuleNVVM<'m> {
    pub functions: FxHashMap<DefId, &'m FunctionNVVM<'m>>,
    pub globals: FxHashMap<DefId, &'m GlobalNVVM>,
    pub types: Vec<TyNVVM<'m>>,
    pub metadata: Metadata<'m>,
    pub arena: &'m Arena<'m>,
}

impl<'m> ModuleNVVM<'m> {
    pub fn new(
        arena: &'m Arena<'m>,
    ) -> Self {
        Self {
            functions: FxHashMap::default(),
            globals: FxHashMap::default(),
            metadata: Metadata {
                version: (1, 0),
                kernel: None,
            },
            types: Vec::new(),
            arena,
        }
    }
    

    /// Assemble the module into a binary representation.
    pub fn assemble(&self) -> Vec<u32> {
        todo!()
    }
    
    pub fn add_function(&mut self, def_id: DefId, function: FunctionNVVM<'m>) {
        let function = self.arena.dropless.alloc(function);
        self.functions.insert(def_id, function);
    }

    pub fn ty_from_type(&mut self, ty: TypeNVVM<'m>) -> TyNVVM<'m> {
        // check if the type already exists
        for t in &self.types {
            if **t == ty {
                return *t;
            }
        }
        let alloc_type = self.arena.dropless.alloc(ty);
        let ty = TyNVVM::new_unchecked(alloc_type);
        self.types.push(ty);
        ty
    }
}

#[derive(Debug)]
pub struct Metadata<'m> {
    version: (u8, u8),
    kernel: Option<&'m FunctionNVVM<'m>>,
}


impl PartialEq for Metadata<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}