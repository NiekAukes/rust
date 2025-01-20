use rustc_data_structures::fx::FxHashMap;

use crate::{module::ModuleNVVM, ty::{TyNVVM, TypeNVVM}, value::{Val, ValueNVVM}};

pub fn declare_intrinsics<'m>(module: ModuleNVVM<'m>) -> ModuleNVVM<'m> {
    let mut module = module;
    let void = module.ty_from_type(TypeNVVM::Zst);
    let trapty = module.ty_from_type(TypeNVVM::Fn(vec![], void));
    let trapval = module.create_val(ValueNVVM::FnRef("llvm.trap".to_string()), Some(trapty));
    module.intrinsics.insert("llvm.trap".to_string(), trapval);
    module
}