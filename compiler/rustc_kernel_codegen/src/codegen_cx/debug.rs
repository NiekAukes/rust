use rustc_codegen_ssa::traits::DebugInfoCodegenMethods;
use rustc_middle::ty::Ty;
use rustc_middle::{mir, ty};
use rustc_target::callconv::FnAbi;

use super::CodegenCx;
use crate::function::FunctionNVVM;
use crate::value::Val;

impl<'tcx, 'm> DebugInfoCodegenMethods<'tcx> for CodegenCx<'m, 'tcx> {
    fn create_function_debug_context(
        &self,
        instance: ty::Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        llfn: &'m FunctionNVVM<'m>,
        mir: &rustc_middle::mir::Body<'tcx>,
    ) -> Option<rustc_codegen_ssa::mir::debuginfo::FunctionDebugContext<'tcx, (), ()>> {
        None
    }

    fn dbg_scope_fn(
        &self,
        instance: ty::Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        maybe_definition_llfn: Option<Self::Function>,
    ) -> Self::DIScope {
    }

    fn dbg_loc(
        &self,
        scope: Self::DIScope,
        inlined_at: Option<Self::DILocation>,
        span: rustc_span::Span,
    ) -> Self::DILocation {
    }

    fn extend_scope_to_file(
        &self,
        scope_metadata: Self::DIScope,
        file: &rustc_span::SourceFile,
    ) -> Self::DIScope {
    }

    fn debuginfo_finalize(&self) {}

    fn create_dbg_var(
        &self,
        variable_name: rustc_span::Symbol,
        variable_type: Ty<'tcx>,
        scope_metadata: Self::DIScope,
        variable_kind: rustc_codegen_ssa::mir::debuginfo::VariableKind,
        span: rustc_span::Span,
    ) -> Self::DIVariable {
    }

    fn create_vtable_debuginfo(
        &self,
        ty: Ty<'tcx>,
        trait_ref: Option<ty::ExistentialTraitRef<'tcx>>,
        vtable: Self::Value,
    ) {
    }
}
