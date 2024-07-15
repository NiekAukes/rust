#![feature(extern_types)]
#![allow(unused)]

use arena::Arena;
use function::FunctionNVVM;
use module::Assemble;
use rustc_data_structures::{intern::Interned, sync::WorkerLocal};
use rustc_middle::query::Providers;
mod builder;
mod codegen_cx;

use rustc_arena::declare_arena;
use ty::{TyNVVM, TypeNVVM};

pub mod base;
pub mod arena;
pub mod module;
pub mod function;
pub mod ty;
pub mod basic_block;
pub mod value;

#[derive(Debug)]
pub struct GlobalNVVM<'m> {
    pub ty: Option<TyNVVM<'m>>,
    pub data: Option<Vec<u8>>,
    pub name: String,
}

impl<'m> GlobalNVVM<'m> {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            ty: None,
            data: Some(data),
            name: String::new(),

        }
    }

    pub fn new_uninitialized(ty: TyNVVM<'m>) -> Self {
        Self {
            ty: Some(ty),
            data: None,
            name: String::new(),
        }
    }
}

pub type Global<'m> = Interned<'m, GlobalNVVM<'m>>;


impl<'m> Assemble<'m> for GlobalNVVM<'m> {
    fn assemble(&self, module: &mut module::ModuleNVVM<'m>) -> String {
        // declare the global
        match self.data {
            Some(ref data) => {
                let ty = self.ty.unwrap_or_else(|| {
                    let ty_i8 = module.ty_from_type(TypeNVVM::I(8));
                    module.ty_from_type(TypeNVVM::Array(ty_i8, data.len()))
                });
                let ty_str = ty.assemble(module);
                format!("@{} = constant {} {:?}", self.name, ty_str, data)
            }
            None => {
                todo!()
            }
        }
    }
}