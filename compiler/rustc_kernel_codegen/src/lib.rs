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
pub mod module;
pub mod ty;
pub mod value;

#[derive(Debug)]
pub struct GlobalNVVM<'m> {
    // initialized global
    // Initialized { ty: Option<TyNVVM<'m>>, data: Option<Vec<u8>>, name: String },

    // // const expressions
    // ConstExpr(ValueNVVM<'m>),
    pub ty: Option<TyNVVM<'m>>,
    pub val: Option<ValueNVVM<'m>>,
    pub name: String,
}

impl<'m> GlobalNVVM<'m> {
    pub fn new(data: Vec<u8>) -> Self {
        let consts = data.iter().map(|x| Const::U8(*x)).collect();
        let val = ValueNVVM::Constant(Const::Arr(consts));
        
        Self { ty: None, val: Some(val), name: String::new() }
        //Self::Initialized { ty: None, data: Some(data), name: String::new() }
    }

    pub fn new_uninitialized(ty: TyNVVM<'m>) -> Self {
        //Self::Initialized { ty: Some(ty), data: None, name: String::new() }
        Self { ty: Some(ty), val: None, name: String::new() }
    }
}

pub type Global<'m> = Interned<'m, GlobalNVVM<'m>>;

impl<'m> Assemble<'m> for GlobalNVVM<'m> {
    fn assemble(&self, module: &mut module::ModuleNVVM<'m>) -> String {
        // declare the global
        // match self {
        //     GlobalNVVM::Initialized { data, name, ty } => match data {
        //         Some(data) => {
        //             let ty = ty.unwrap_or_else(|| {
        //                 let ty_i8 = module.ty_from_type(TypeNVVM::I(8));
        //                 module.ty_from_type(TypeNVVM::Array(ty_i8, data.len()))
        //             });
        //             let ty_str = ty.assemble(module);
        //             let data_str = assemble_array_decl(module, data);
        //             format!("@{} = constant {} {}", name, ty_str, data_str)
        //         }
        //         None => {
        //             todo!()
        //         }
        //     },

        //     GlobalNVVM::ConstExpr(value) => {
        //         todo!()
        //     }
        // }
        match self.val {
            Some(ref val) => {
                let (data_str, sz) = match val {
                    ValueNVVM::Constant(c) => (c.assemble_for_const(module), c.size()),
                    //ValueNVVM::Instr(i) => i.assemble(module),
                    _ => todo!(),
                };
                let ty = self.ty.unwrap_or_else(|| {
                    let ty_i8 = module.ty_from_type(TypeNVVM::I(8));
                    // try to 
                    module.ty_from_type(TypeNVVM::Array(ty_i8, sz))
                });
                let ty_str = ty.assemble(module);
                
                format!("@{} = constant {} {}", self.name, ty_str, data_str)
            }
            None => {
                // DUMMY
                format!("@{} = dummy global", self.name)            
            }
        }
    }
}

fn assemble_array_decl<'m>(module: &mut module::ModuleNVVM<'m>, data: &[u8]) -> String {
    // should be [i8 0, i8 1, i8 2, i8 3, i8 4, i8 5, i8 6, i8 7, i8 8, i8 9] etc.
    let data_str = data.iter().map(|d| format!("i8 {}", d)).collect::<Vec<_>>().join(", ");
    format!("[{}]", data_str)
}
