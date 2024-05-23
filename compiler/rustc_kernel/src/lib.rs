use rustc_hir::def_id::DefId;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrFlags;
use rustc_middle::mir::interpret::ConstAllocation;
use rustc_middle::mir::Body;
use rustc_middle::ty::{Instance, TyCtxt};
use rustc_middle::query::Providers;
use rustc_middle::bug;
use rustc_hir::def::DefKind;
use rustc_middle::mir::mono::{CodegenUnit, Linkage, MonoItem, MonoItemData, Visibility};

mod kernel_embedder;

#[macro_use]
extern crate tracing;

pub fn is_kernel<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> bool {
    // the def_id is a kernel if it is function-like and has a kernel flag
    if tcx.def_kind(def_id) != DefKind::Fn {
        return false;
    }

    let fn_attrs = tcx.codegen_fn_attrs(def_id).flags;
    fn_attrs.contains(CodegenFnAttrFlags::KERNEL)
}

pub fn compile_kernel<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> &'tcx Body<'tcx> {
    // TODO! compile the kernel module
    let code = b"1234";
    let constant = kernel_embedder::embed_kernel(tcx, code);
    tcx.arena.alloc(constant)
}

pub fn provide(providers: &mut Providers) {
    providers.processed_kernel_mir = compile_kernel;

    providers.is_kernel = is_kernel;
}
