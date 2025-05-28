use rustc_data_structures::intern::Interned;

use crate::{
    module::{self, Assemble},
    ty::{TyNVVM, TypeNVVM},
    value::{Const, Val, ValueNVVM},
};

#[derive(Debug)]
pub struct GlobalNVVM<'m> {
    // initialized global
    // Initialized { ty: Option<TyNVVM<'m>>, data: Option<Vec<u8>>, name: String },

    // // const expressions
    // ConstExpr(ValueNVVM<'m>),
    pub val: Val<'m>,
    pub name: String,
}
impl<'m> GlobalNVVM<'m> {
    pub fn new(val: Val<'m>) -> Self {
        // let consts = data.iter().map(|x| Const::U8(*x)).collect();
        // let val = (Const::Arr(consts));

        Self { val, name: String::new() }
    }
}

impl<'m> Assemble<'m> for GlobalNVVM<'m> {
    fn assemble(&self, module: &mut module::ModuleNVVM<'m>) -> String {
        match self.val.0 {
            ValueNVVM::Constant(ref c) => {
                let data_str = c.assemble_for_const(module);
                let sz = c.size();
                // let ty = self.ty.unwrap_or_else(|| {
                //     let ty_i8 = module.ty_from_type(TypeNVVM::I(8));
                //     module.ty_from_type(TypeNVVM::Array(ty_i8, sz))
                // });
                let ty = *module.valtypes.get(&self.val).expect("GlobalNVVM must have a type");
                let ty_str = ty.assemble(module);

                format!("@{} = constant {}", self.name, data_str)
            }
            ValueNVVM::ConstExpr(ref expr) => {
                todo!()
            }
            _ => {
                panic!(
                    "GlobalNVVM must be a constant or const expression, found: {:?}",
                    self.val.0
                );
            }
        }
    }
}

#[derive(Debug)]
pub enum ConstExpr<'m> {
    GEP { ty: TyNVVM<'m>, val: Val<'m>, indices: Vec<Const> },
}
