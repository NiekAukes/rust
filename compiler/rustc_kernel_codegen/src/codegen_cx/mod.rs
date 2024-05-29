use rustc_codegen_ssa::traits::AsmMethods;
use rustc_codegen_ssa::traits::BackendTypes;
use rustc_codegen_ssa::traits::CodegenMethods;
use rustc_codegen_ssa::traits::DebugInfoMethods;
use rustc_middle::ty::layout::HasParamEnv;
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::TyCtxt;
use rustc_target::spec::HasTargetSpec;
use crate::llvm::Type;
use crate::llvm::Value;
use crate::llvm::BasicBlock;

mod declare;
mod consts;
mod statics;
mod debug;
mod misc;
mod abi;

pub struct CodegenCx<'ll, 'tcx> {
    // ...
    dummy: &'ll Value,
    tcx: TyCtxt<'tcx>,
}

impl<'ll> BackendTypes for CodegenCx<'ll, '_> {
    type Value = &'ll Value;
    // FIXME(eddyb) replace this with a `Function` "subclass" of `Value`.
    type Function = &'ll Value;

    type BasicBlock = &'ll BasicBlock;
    type Type = &'ll Type;
    type Funclet = ();

    type DIScope = ();
    type DILocation = ();
    type DIVariable = ();
}

//impl<'tcx> CodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {}

impl HasTargetSpec for CodegenCx<'_, '_> {
    fn target_spec(&self) -> &rustc_target::spec::Target {
        todo!()
    }
}

impl<'tcx> AsmMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn codegen_global_asm(
        &self,
        template: &[rustc_ast::InlineAsmTemplatePiece],
        operands: &[rustc_codegen_ssa::traits::GlobalAsmOperandRef<'tcx>],
        options: rustc_ast::InlineAsmOptions,
        line_spans: &[rustc_span::Span],
    ) {
        todo!()
    }
}


impl<'tcx> HasParamEnv<'tcx> for CodegenCx<'_, 'tcx> {
    fn param_env(&self) -> rustc_middle::ty::ParamEnv<'tcx> {
        todo!()
    }
}

impl<'tcx> HasTyCtxt<'tcx> for CodegenCx<'_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}