use rustc_codegen_ssa::traits::{BaseTypeMethods, ConstMethods};
use rustc_middle::mir::interpret::{AllocId, AllocRange, ConstAllocation, GlobalAlloc, Scalar};
use rustc_target::abi::{self, Size};

use crate::{
    ty::TyNVVM,
    value::{Const, Val, ValueNVVM},
    GlobalNVVM,
};

use super::CodegenCx;

impl<'tcx> ConstMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn const_null(&self, t: Self::Type) -> Self::Value {
        todo!()
    }

    fn const_undef(&self, t: Self::Type) -> Self::Value {
        let value = ValueNVVM::Constant(Const::Undef);
        self.get_module_mut().create_val(value, Some(t))
    }

    fn const_poison(&self, t: Self::Type) -> Self::Value {
        self.const_undef(t)
    }

    fn const_int(&self, t: Self::Type, i: i64) -> Self::Value {
        todo!()
    }

    fn const_uint(&self, t: Self::Type, i: u64) -> Self::Value {
        self.const_u64(i)
    }

    fn const_uint_big(&self, t: Self::Type, u: u128) -> Self::Value {
        self.const_u128(u)
    }

    fn const_bool(&self, val: bool) -> Self::Value {
        let value = ValueNVVM::Constant(Const::Bool(val));
        self.get_module_mut().create_val(value, Some(self.type_i1()))
    }

    fn const_i16(&self, i: i16) -> Self::Value {
        todo!()
    }

    fn const_i32(&self, i: i32) -> Self::Value {
        let value = ValueNVVM::Constant(Const::I32(i));
        let ty = self.type_i32();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_i8(&self, i: i8) -> Self::Value {
        let value = ValueNVVM::Constant(Const::I8(i));
        let ty = self.type_i8();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_u32(&self, i: u32) -> Self::Value {
        let value = ValueNVVM::Constant(Const::U32(i));
        let ty = self.type_i32();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_u64(&self, i: u64) -> Self::Value {
        let value = ValueNVVM::Constant(Const::U64(i));
        let ty = self.type_i64();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_u128(&self, i: u128) -> Self::Value {
        // TODO: Implement
        let value = ValueNVVM::Constant(Const::U64(i as u64));
        let ty = self.type_i64();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_usize(&self, i: u64) -> Self::Value {
        self.const_u64(i as u64)
    }

    fn const_u8(&self, i: u8) -> Self::Value {
        todo!()
    }

    fn const_real(&self, t: Self::Type, val: f64) -> Self::Value {
        todo!()
    }

    fn const_str(&self, s: &str) -> (Self::Value, Self::Value) {
        todo!()
    }

    fn const_struct(&self, elts: &[Self::Value], packed: bool) -> Self::Value {
        todo!()
    }

    fn const_to_opt_uint(&self, v: Self::Value) -> Option<u64> {
        // if the value is a constant, we can extract the value
        // otherwise we can't
        // this is for optimization purposes
        match *v {
            ValueNVVM::Constant(Const::U8(u)) => Some(u as u64),
            ValueNVVM::Constant(Const::U16(u)) => Some(u as u64),
            ValueNVVM::Constant(Const::U32(u)) => Some(u as u64),
            ValueNVVM::Constant(Const::U64(u)) => Some(u),
            ValueNVVM::Constant(Const::U128(u)) => None,
            ValueNVVM::Constant(Const::I8(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }
            ValueNVVM::Constant(Const::I16(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }
            ValueNVVM::Constant(Const::I32(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }
            ValueNVVM::Constant(Const::I64(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    fn const_to_opt_u128(&self, v: Self::Value, sign_ext: bool) -> Option<u128> {
        //println!("TODO: const_to_opt_u128\n\n");
        //println!("Value: {:?} sign_ext: {}", v, sign_ext);
        None // Not supported
    }

    fn const_data_from_alloc(
        &self,
        alloc: rustc_middle::mir::interpret::ConstAllocation<'tcx>,
    ) -> Self::Value {
        let module = self.get_module_mut();
        let bytes = alloc.inner().get_bytes_unchecked(AllocRange {
            start: Size::from_bytes(0),
            size: Size::from_bytes(alloc.inner().len()),
        });
        let allocation = GlobalNVVM::new(bytes.to_vec());
        let value = module.add_allocation(allocation);
        value
    }

    fn scalar_to_backend(
        &self,
        cv: rustc_middle::mir::interpret::Scalar,
        layout: rustc_target::abi::Scalar,
        llty: Self::Type,
    ) -> Self::Value {
        let value = match cv {
            Scalar::Int(i) => {
                let sz = i.size().bytes();
                match sz {
                    1 => self.const_i8(i.try_to_i8().unwrap()),
                    2 => self.const_i16(i.try_to_i16().unwrap()),
                    4 => self.const_i32(i.try_to_i32().unwrap()),
                    8 => self.const_i64(i.try_to_i64().unwrap()),
                    16 => self.const_i64(i.try_to_i128().unwrap() as i64),
                    _ => todo!(),
                }
            }
            Scalar::Ptr(p, _) => {
                let (prov, offset) = p.into_parts();
                match self.tcx.global_alloc(prov.alloc_id()) {
                    GlobalAlloc::Memory(alloc) => {
                        return self.const_data_from_alloc(alloc);
                    }
                    GlobalAlloc::Function(instance) => todo!(),
                    GlobalAlloc::Static(def_id) => {
                        let alloc = self.tcx.eval_static_initializer(def_id).unwrap();
                        return self.const_data_from_alloc(alloc);
                    }
                    GlobalAlloc::VTable(_, _) => todo!(),
                }
                todo!()
            }
        };

        value
    }

    fn const_bitcast(&self, val: Self::Value, ty: Self::Type) -> Self::Value {
        //self.
        todo!()
    }

    fn const_ptr_byte_offset(
        &self,
        val: Self::Value,
        offset: rustc_target::abi::Size,
    ) -> Self::Value {
        todo!()
    }
}

impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    pub fn const_i64(&self, i: i64) -> Val<'m> {
        let value = ValueNVVM::Constant(Const::I64(i));
        let ty = self.type_i64();
        self.get_module_mut().create_val(value, Some(ty))
    }
}
