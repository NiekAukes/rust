use rustc_abi::HasDataLayout;
use rustc_codegen_ssa::traits::{BackendTypes, BuilderMethods, StaticBuilderMethods};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_target::spec::{HasTargetSpec, Target};

use crate::codegen_cx::CodegenCx;

mod abi;
mod asm;
mod debug_info;
mod intrinsic;
mod methods;
pub struct Builder<'a, 'm, 'tcx> {
    codegen_cx: &'a CodegenCx<'m, 'tcx>,
    basic_block: &'m crate::basic_block::BasicBlock<'m>,
}

impl<'a, 'm, 'tcx> HasTargetSpec for Builder<'a, 'm, 'tcx> {
    fn target_spec(&self) -> &Target {
        todo!()
    }
}

impl StaticBuilderMethods for Builder<'_, '_, '_> {
    fn get_static(&mut self, def_id: rustc_hir::def_id::DefId) -> Self::Value {
        todo!()
    }
}

impl<'m, 'tcx> BackendTypes for Builder<'_, 'm, 'tcx> {
    type Value = <CodegenCx<'m, 'tcx> as BackendTypes>::Value;
    type Function = <CodegenCx<'m, 'tcx> as BackendTypes>::Function;
    type BasicBlock = <CodegenCx<'m, 'tcx> as BackendTypes>::BasicBlock;
    type Type = <CodegenCx<'m, 'tcx> as BackendTypes>::Type;
    type Funclet = <CodegenCx<'m, 'tcx> as BackendTypes>::Funclet;

    type DIScope = <CodegenCx<'m, 'tcx> as BackendTypes>::DIScope;
    type DILocation = <CodegenCx<'m, 'tcx> as BackendTypes>::DILocation;
    type DIVariable = <CodegenCx<'m, 'tcx> as BackendTypes>::DIVariable;

    type Metadata = ();
}

impl<'tcx> HasTyCtxt<'tcx> for Builder<'_, '_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.codegen_cx.tcx()
    }
}

impl HasDataLayout for Builder<'_, '_, '_> {
    fn data_layout(&self) -> &rustc_abi::TargetDataLayout {
        self.codegen_cx.data_layout()
    }
}

impl<'m, 'tcx> std::ops::Deref for Builder<'_, 'm, 'tcx> {
    type Target = CodegenCx<'m, 'tcx>;

    fn deref(&self) -> &Self::Target {
        self.codegen_cx
    }
}
