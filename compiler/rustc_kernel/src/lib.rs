use rustc_hir::def::DefKind;
use rustc_hir::def_id::{CrateNum, DefId};
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrFlags;
use rustc_middle::mir::Body;
use rustc_middle::query::Providers;
use rustc_middle::ty::TyCtxt;
use rustc_mir_build::thir::print;
use rustc_span::symbol::sym;

pub mod codegen;
pub mod kernel_embedder;

#[macro_use]
extern crate tracing;

pub fn is_kernel<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> bool {
    // the def_id is a kernel if it is function-like and has a kernel flag
    if tcx.def_kind(def_id) != DefKind::Fn {
        return false;
    }

    let fn_attrs = tcx.codegen_fn_attrs(def_id).flags;
    let a = fn_attrs.contains(CodegenFnAttrFlags::KERNEL);
    a
}

pub fn kernel_allocator_provider(tcx: TyCtxt<'_>, _krate: CrateNum) -> Option<DefId> {
    let mut allocator = None;

    for item_id in tcx.hir_body_owners() {
        let def_id = item_id.to_def_id();

        if tcx.has_attr(def_id, sym::kernel_allocator) {
            if allocator.is_some() {
                panic!("multiple `#[kernel_allocator]` items found in crate");
            }
            allocator = Some(def_id);
        }
    }
    allocator
}

pub fn compile_kernel<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> &'tcx Body<'tcx> {
    let (name, code) = codegen::generate(tcx, def_id);
    let constant = kernel_embedder::embed_kernel(tcx, def_id, &name, code.as_slice());
    tcx.arena.alloc(constant)
}

pub fn provide(providers: &mut Providers) {
    providers.processed_kernel_mir = compile_kernel;
    providers.kernel_allocator = kernel_allocator_provider;

    providers.is_kernel = is_kernel;
}
