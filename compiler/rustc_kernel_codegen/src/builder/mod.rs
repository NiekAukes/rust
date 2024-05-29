use rustc_middle::ty::{layout::{HasParamEnv, HasTyCtxt}, TyCtxt};
use rustc_codegen_ssa::traits::{BackendTypes, BuilderMethods, HasCodegen, StaticBuilderMethods};
use rustc_target::{abi::HasDataLayout, spec::{HasTargetSpec, Target}};
use crate::codegen_cx::CodegenCx;

mod methods;
mod asm;
mod abi;
mod intrinsic;
mod debug_info;
pub struct Builder<'a, 'll, 'tcx> {
    codegen_cx: &'a mut CodegenCx<'ll, 'tcx>,
    tcx: TyCtxt<'tcx>,
}

impl<'a, 'll, 'tcx> HasTargetSpec for Builder<'a, 'll, 'tcx> {
    fn target_spec(&self) -> &Target {
        todo!()
    }
}

impl StaticBuilderMethods for Builder<'_, '_, '_> {
    fn get_static(&mut self, def_id: rustc_hir::def_id::DefId) -> Self::Value {
        todo!()
    }
}

impl<'ll, 'tcx> BackendTypes for Builder<'_, 'll, 'tcx> {
    type Value = <CodegenCx<'ll, 'tcx> as BackendTypes>::Value;
    type Function = <CodegenCx<'ll, 'tcx> as BackendTypes>::Function;
    type BasicBlock = <CodegenCx<'ll, 'tcx> as BackendTypes>::BasicBlock;
    type Type = <CodegenCx<'ll, 'tcx> as BackendTypes>::Type;
    type Funclet = <CodegenCx<'ll, 'tcx> as BackendTypes>::Funclet;

    type DIScope = <CodegenCx<'ll, 'tcx> as BackendTypes>::DIScope;
    type DILocation = <CodegenCx<'ll, 'tcx> as BackendTypes>::DILocation;
    type DIVariable = <CodegenCx<'ll, 'tcx> as BackendTypes>::DIVariable;
}

impl<'ll, 'tcx> HasCodegen<'tcx> for Builder<'_, 'll, 'tcx> {
    type CodegenCx = CodegenCx<'ll, 'tcx>;
}

impl<'tcx> HasParamEnv<'tcx> for Builder<'_, '_, 'tcx> {
    fn param_env(&self) -> rustc_middle::ty::ParamEnv<'tcx> {
        todo!()
    }
}

impl<'tcx> HasTyCtxt<'tcx> for Builder<'_, '_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}

impl HasDataLayout for Builder<'_, '_, '_> {
    fn data_layout(&self) -> &rustc_target::abi::TargetDataLayout {
        todo!()
    }
}


impl<'ll, 'tcx> std::ops::Deref for Builder<'_, 'll, 'tcx> {
    type Target = CodegenCx<'ll, 'tcx>;

    fn deref(&self) -> &Self::Target {
        self.codegen_cx
    }
}