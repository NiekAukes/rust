#![feature(extern_types)]
#![allow(unused)]

use arena::Arena;
use rustc_data_structures::sync::WorkerLocal;
use rustc_middle::query::Providers;
mod builder;
mod codegen_cx;

use rustc_arena::declare_arena;

pub mod base;
pub mod arena;

pub struct ModuleNVVM<'m> {
    pub functions: Vec<&'m FunctionNVVM>,
    pub globals: Vec<&'m GlobalNVVM>,
    pub types: Vec<&'m TypeNVVM>,
    pub metadata: Metadata<'m>,
    pub arena: &'m Arena<'m>,
}

impl<'m> ModuleNVVM<'m> {
    pub fn new(
        arena: &'m Arena<'m>,
    ) -> Self {
        Self {
            functions: Vec::new(),
            globals: Vec::new(),
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

    pub fn add_function(&mut self, function: FunctionNVVM) -> &'m FunctionNVVM {
        let function = self.arena.dropless.alloc(function);
        self.functions.push(function);
        function
    }
}
#[derive(Debug)]
pub struct FunctionNVVM {
    pub is_kernel: bool,
    pub name: String,
    pub ret: Option<TyNVVM>,
    pub args: Vec<TyNVVM>,
}
#[derive(Debug)]
pub struct GlobalNVVM;
#[derive(Debug)]
pub struct Metadata<'m> {
    version: (u8, u8),
    kernel: Option<&'m FunctionNVVM>,
}

#[derive(Debug)]
pub struct TypeNVVM;

/// Interned type for NVVM.
#[derive(Debug)]
pub struct TyNVVM; 

pub struct Value;
pub struct BasicBlock;

impl PartialEq for Metadata<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl PartialEq for FunctionNVVM {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

