#![feature(extern_types)]
#![allow(unused)]

use arena::Arena;
use rustc_data_structures::{intern::Interned, sync::WorkerLocal};
use rustc_middle::query::Providers;
mod builder;
mod codegen_cx;

use rustc_arena::declare_arena;

pub mod base;
pub mod arena;
pub mod module;
pub mod function;
pub mod ty;

#[derive(Debug)]
pub struct GlobalNVVM;

#[derive(Debug)]
pub struct Value;

#[derive(Debug)]
pub struct BasicBlock;

impl PartialEq for GlobalNVVM {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}