use rustc_codegen_ssa::traits::ConstMethods;

use super::CodegenCx;

impl<'tcx> ConstMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn const_null(&self, t: Self::Type) -> Self::Value {
        todo!()
    }

    fn const_undef(&self, t: Self::Type) -> Self::Value {
        todo!()
    }

    fn const_poison(&self, t: Self::Type) -> Self::Value {
        todo!()
    }

    fn const_int(&self, t: Self::Type, i: i64) -> Self::Value {
        todo!()
    }

    fn const_uint(&self, t: Self::Type, i: u64) -> Self::Value {
        todo!()
    }

    fn const_uint_big(&self, t: Self::Type, u: u128) -> Self::Value {
        todo!()
    }

    fn const_bool(&self, val: bool) -> Self::Value {
        todo!()
    }

    fn const_i16(&self, i: i16) -> Self::Value {
        todo!()
    }

    fn const_i32(&self, i: i32) -> Self::Value {
        todo!()
    }

    fn const_i8(&self, i: i8) -> Self::Value {
        todo!()
    }

    fn const_u32(&self, i: u32) -> Self::Value {
        todo!()
    }

    fn const_u64(&self, i: u64) -> Self::Value {
        todo!()
    }

    fn const_u128(&self, i: u128) -> Self::Value {
        todo!()
    }

    fn const_usize(&self, i: u64) -> Self::Value {
        todo!()
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
        todo!()
    }

    fn const_to_opt_u128(&self, v: Self::Value, sign_ext: bool) -> Option<u128> {
        todo!()
    }

    fn const_data_from_alloc(&self, alloc: rustc_middle::mir::interpret::ConstAllocation<'tcx>) -> Self::Value {
        todo!()
    }

    fn scalar_to_backend(&self, cv: rustc_middle::mir::interpret::Scalar, layout: rustc_target::abi::Scalar, llty: Self::Type) -> Self::Value {
        todo!()
    }

    fn const_bitcast(&self, val: Self::Value, ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn const_ptr_byte_offset(&self, val: Self::Value, offset: rustc_target::abi::Size) -> Self::Value {
        todo!()
    }
}