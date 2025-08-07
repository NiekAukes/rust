use rustc_data_structures::intern::Interned;
use rustc_middle::ty;

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
                let expr_str = expr.assemble(module);
                let ty = *module.valtypes.get(&self.val).expect("GlobalNVVM must have a type");
                let ty_str = ty.assemble(module);

                format!("@{} = constant {} {}", self.name, ty_str, expr_str)
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
    GEP { ty: TyNVVM<'m>, val: Val<'m>, indices: Vec<Const<'m>> },
    BitCast { ty: TyNVVM<'m>, val: Val<'m> },
}

impl<'m> ConstExpr<'m> {
    pub fn assemble(&self, module: &mut module::ModuleNVVM<'m>) -> String {
        match self {
            ConstExpr::GEP { ty, val, indices } => {
                let val_str = assemble_const_val(val, module);
                let val_ty = *module.valtypes.get(val).expect("ConstExpr GEP must have a type");
                let val_tystr = val_ty.assemble(module);
                let indices_str: Vec<String> =
                    indices.iter().map(|i| format!("i64 {}", i.assemble(module))).collect();
                format!(
                    "getelementptr ({}, {} {}, {})",
                    ty.assemble(module),
                    val_tystr,
                    val_str,
                    indices_str.join(", ")
                )
            }
            ConstExpr::BitCast { ty, val } => {
                let val_str = assemble_const_val(val, module);
                let val_ty = *module.valtypes.get(val).expect("ConstExpr BitCast must have a type");
                let val_tystr = val_ty.assemble(module);
                let ty_str = ty.assemble(module);
                format!("bitcast ({} {} to {})", val_tystr, val_str, ty_str)
            }
        }
    }
}

fn assemble_const_val<'m>(val: &Val<'m>, module: &mut module::ModuleNVVM<'m>) -> String {
    match val.0 {
        ValueNVVM::Constant(ref c) => c.assemble_for_const(module),
        //ValueNVVM::Global(GlobalNVVM { ref val, .. }) => assemble_const_val(val, module),
        ValueNVVM::Global(GlobalNVVM { ref val, name }) => {
            //let ty = *module.valtypes.get(val).expect("GlobalNVVM must have a type");
            format!("@{}", name)
        }
        ValueNVVM::ConstExpr(expr) => expr.assemble(module),
        _ => panic!("Expected a constant value, found: {:?}", val.0),
    }
}
