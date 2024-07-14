#![feature(extern_types)]
#![allow(unused)]

use arena::Arena;
use function::FunctionNVVM;
use rustc_data_structures::{intern::Interned, sync::WorkerLocal};
use rustc_middle::query::Providers;
mod builder;
mod codegen_cx;

use rustc_arena::declare_arena;
use ty::TyNVVM;

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
    pub data: Vec<u8>,
    pub name: String,
}

impl<'m> GlobalNVVM<'m> {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            ty: None,
            data,
            name: String::new(),
        }
    }
}

pub type Global<'m> = Interned<'m, GlobalNVVM<'m>>;
