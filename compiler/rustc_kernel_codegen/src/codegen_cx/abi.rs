use rustc_codegen_ssa::traits::{BaseTypeMethods, LayoutTypeMethods, TypeMembershipMethods};
use rustc_middle::ty::{layout::{FnAbiOfHelpers, LayoutOfHelpers}, Ty};
use rustc_target::abi::HasDataLayout;

use super::CodegenCx;

impl<'tcx> TypeMembershipMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn add_type_metadata(&self, _function: Self::Function, _typeid: String) {}

    fn set_type_metadata(&self, _function: Self::Function, _typeid: String) {}

    fn typeid_metadata(&self, _typeid: String) -> Option<Self::Value> {
        None
    }

    fn add_kcfi_type_metadata(&self, _function: Self::Function, _typeid: u32) {}

    fn set_kcfi_type_metadata(&self, _function: Self::Function, _typeid: u32) {}
}

impl<'tcx> FnAbiOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    type FnAbiOfResult = &'tcx rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>;

    fn handle_fn_abi_err(
        &self,
        err: rustc_middle::ty::layout::FnAbiError<'tcx>,
        span: rustc_span::Span,
        fn_abi_request: rustc_middle::ty::layout::FnAbiRequest<'tcx>,
    ) -> <Self::FnAbiOfResult as rustc_middle::ty::layout::MaybeResult<&'tcx rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>>>::Error {
        todo!()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    type LayoutOfResult = rustc_middle::ty::layout::TyAndLayout<'tcx>;

    fn handle_layout_err(
        &self,
        err: rustc_middle::ty::layout::LayoutError<'tcx>,
        span: rustc_span::Span,
        ty: Ty<'tcx>,
    ) -> <Self::LayoutOfResult as rustc_middle::ty::layout::MaybeResult<rustc_middle::ty::layout::TyAndLayout<'tcx>>>::Error {
        todo!()
    }
}

impl HasDataLayout for CodegenCx<'_, '_> {
    fn data_layout(&self) -> &rustc_target::abi::TargetDataLayout {
        &self.tcx.data_layout
    }
}

impl<'tcx> LayoutTypeMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn backend_type(&self, layout: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> Self::Type {
        todo!()
    }

    fn cast_backend_type(&self, ty: &rustc_target::abi::call::CastTarget) -> Self::Type {
        todo!()
    }

    fn fn_decl_backend_type(&self, fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!()
    }

    fn fn_ptr_backend_type(&self, fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!()
    }

    fn reg_backend_type(&self, ty: &rustc_target::abi::call::Reg) -> Self::Type {
        todo!()
    }

    fn immediate_backend_type(&self, layout: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> Self::Type {
        todo!()
    }

    fn is_backend_immediate(&self, layout: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> bool {
        todo!()
    }

    fn is_backend_scalar_pair(&self, layout: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> bool {
        todo!()
    }

    fn scalar_pair_element_backend_type(
        &self,
        layout: rustc_middle::ty::layout::TyAndLayout<'tcx>,
        index: usize,
        immediate: bool,
    ) -> Self::Type {
        todo!()
    }
}

impl<'tcx> BaseTypeMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn type_i1(&self) -> Self::Type {
        todo!()
    }

    fn type_i8(&self) -> Self::Type {
        todo!()
    }

    fn type_i16(&self) -> Self::Type {
        todo!()
    }

    fn type_i32(&self) -> Self::Type {
        todo!()
    }

    fn type_i64(&self) -> Self::Type {
        todo!()
    }

    fn type_i128(&self) -> Self::Type {
        todo!()
    }

    fn type_isize(&self) -> Self::Type {
        todo!()
    }

    fn type_f16(&self) -> Self::Type {
        todo!()
    }

    fn type_f32(&self) -> Self::Type {
        todo!()
    }

    fn type_f64(&self) -> Self::Type {
        todo!()
    }

    fn type_f128(&self) -> Self::Type {
        todo!()
    }

    fn type_array(&self, ty: Self::Type, len: u64) -> Self::Type {
        todo!()
    }

    fn type_func(&self, args: &[Self::Type], ret: Self::Type) -> Self::Type {
        todo!()
    }

    fn type_struct(&self, els: &[Self::Type], packed: bool) -> Self::Type {
        todo!()
    }

    fn type_kind(&self, ty: Self::Type) -> rustc_codegen_ssa::common::TypeKind {
        todo!()
    }

    fn type_ptr(&self) -> Self::Type {
        todo!()
    }

    fn type_ptr_ext(&self, address_space: rustc_target::abi::AddressSpace) -> Self::Type {
        todo!()
    }

    fn element_type(&self, ty: Self::Type) -> Self::Type {
        todo!()
    }

    fn vector_length(&self, ty: Self::Type) -> usize {
        todo!()
    }

    fn float_width(&self, ty: Self::Type) -> usize {
        todo!()
    }

    fn int_width(&self, ty: Self::Type) -> u64 {
        todo!()
    }

    fn val_ty(&self, v: Self::Value) -> Self::Type {
        todo!()
    }
}