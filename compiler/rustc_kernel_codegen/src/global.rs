use rustc_data_structures::intern::Interned;

use crate::{
    module::{self, Assemble},
    ty::{TyNVVM, TypeNVVM},
    value::{Const, ValueNVVM},
};

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
                todo!()
            }
        }
    }
}
