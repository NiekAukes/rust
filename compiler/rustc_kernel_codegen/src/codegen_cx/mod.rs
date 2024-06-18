use std::cell::UnsafeCell;
use std::collections::HashMap;

use rustc_codegen_ssa::traits::AsmMethods;
use rustc_codegen_ssa::traits::BackendTypes;
use rustc_codegen_ssa::traits::CodegenMethods;
use rustc_codegen_ssa::traits::DebugInfoMethods;
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::intern::Interned;
use rustc_middle::ty::layout::HasParamEnv;
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::Ty;
use rustc_middle::ty::TyCtxt;
use rustc_target::abi::VariantIdx;
use rustc_target::spec::HasTargetSpec;
use crate::function::FunctionNVVM;
use crate::module::ModuleNVVM;
use crate::ty::{TyNVVM, TypeNVVM};
use crate::Value;
use crate::BasicBlock;

mod declare;
mod consts;
mod statics;
mod debug;
mod misc;
mod abi;

pub struct CodegenCx<'m, 'tcx> {
    // ...
    tcx: TyCtxt<'tcx>,
    // unsafe cell to allow mutation of the module
    module: UnsafeCell<ModuleNVVM<'m>>,

    pub(crate) typecache: UnsafeCell<FxHashMap<(Ty<'tcx>, Option<VariantIdx>), TyNVVM<'m>>>,
}

impl<'m> BackendTypes for CodegenCx<'m, '_> {
    type Value = &'m Value;
    // FIXME(eddyb) replace this with a `Function` "subclass" of `Value`.
    type Function = &'m FunctionNVVM<'m>;

    type BasicBlock = &'m BasicBlock;
    type Type = TyNVVM<'m>;
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
        rustc_middle::ty::ParamEnv::reveal_all()
    }
}

impl<'tcx> HasTyCtxt<'tcx> for CodegenCx<'_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}


impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, module: ModuleNVVM<'m>) -> Self {
        Self {
            // ...
            tcx,
            module: UnsafeCell::new(module),
            typecache: UnsafeCell::new(FxHashMap::default()),
        }
    }

    pub fn finalize(self) -> ModuleNVVM<'m> {
        self.module.into_inner()
    }
}