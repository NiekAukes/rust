#![feature(extern_types)]
#![allow(unused)]
#![feature(map_try_insert)]

use arena::Arena;
use function::FunctionNVVM;
use module::Assemble;
use rustc_data_structures::{intern::Interned, sync::WorkerLocal};
use rustc_middle::query::Providers;
mod builder;
mod codegen_cx;

use rustc_arena::declare_arena;
use ty::{TyNVVM, TypeNVVM};
use value::{Const, ValueNVVM};

pub mod arena;
pub mod base;
pub mod basic_block;
pub mod function;
pub mod global;
pub mod module;
pub mod ty;
pub mod value;
