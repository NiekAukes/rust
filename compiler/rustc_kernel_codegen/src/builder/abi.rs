use rustc_codegen_ssa::traits::{AbiBuilderMethods, ArgAbiMethods};
use rustc_middle::ty::{layout::{FnAbiOfHelpers, LayoutOfHelpers}, Ty};

use super::Builder;

impl<'tcx> AbiBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn get_param(&mut self, index: usize) -> Self::Value {
        todo!()
    }
}

impl<'tcx> ArgAbiMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn store_fn_arg(
        &mut self,
        arg_abi: &rustc_target::abi::call::ArgAbi<'tcx, Ty<'tcx>>,
        idx: &mut usize,
        dst: rustc_codegen_ssa::mir::place::PlaceRef<'tcx, Self::Value>,
    ) {
        todo!()
    }

    fn store_arg(
        &mut self,
        arg_abi: &rustc_target::abi::call::ArgAbi<'tcx, Ty<'tcx>>,
        val: Self::Value,
        dst: rustc_codegen_ssa::mir::place::PlaceRef<'tcx, Self::Value>,
    ) {
        todo!()
    }

    fn arg_memory_ty(&self, arg_abi: &rustc_target::abi::call::ArgAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!()
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
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

impl<'tcx> LayoutOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
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