use rustc_codegen_ssa::traits::DebugInfoMethods;
use rustc_middle::ty::Ty;
use rustc_middle::ty;
use rustc_middle::mir;
use rustc_target::abi::call::FnAbi;

use super::CodegenCx;

impl<'tcx> DebugInfoMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn create_vtable_debuginfo(
        &self,
        ty: Ty<'tcx>,
        trait_ref: Option<ty::PolyExistentialTraitRef<'tcx>>,
        vtable: Self::Value,
    ) {
        todo!()
    }

    fn create_function_debug_context(
        &self,
        instance: ty::Instance<'tcx>,
        fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>,
        llfn: Self::Function,
        mir: &rustc_middle::mir::Body<'tcx>,
    ) -> Option<rustc_codegen_ssa::mir::debuginfo::FunctionDebugContext<'tcx, Self::DIScope, Self::DILocation>> {
        todo!()
    }

    fn dbg_scope_fn(
        &self,
        instance: ty::Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        maybe_definition_llfn: Option<Self::Function>,
    ) -> Self::DIScope {
        todo!()
    }

    fn dbg_loc(
        &self,
        scope: Self::DIScope,
        inlined_at: Option<Self::DILocation>,
        span: rustc_span::Span,
    ) -> Self::DILocation {
        todo!()
    }

    fn extend_scope_to_file(
        &self,
        scope_metadata: Self::DIScope,
        file: &rustc_span::SourceFile,
    ) -> Self::DIScope {
        todo!()
    }

    fn debuginfo_finalize(&self) {
        todo!()
    }

    fn create_dbg_var(
        &self,
        variable_name: rustc_span::Symbol,
        variable_type: Ty<'tcx>,
        scope_metadata: Self::DIScope,
        variable_kind: rustc_codegen_ssa::mir::debuginfo::VariableKind,
        span: rustc_span::Span,
    ) -> Self::DIVariable {
        todo!()
    }
}