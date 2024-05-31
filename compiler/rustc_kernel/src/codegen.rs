use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

pub fn generate<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> Vec<u8> {
    // we need to generate the code for the kernel
    // to do that, we need to generate a cgu for the kernel
    let cgu = tcx.collect_mono_items_for_def(def_id);

    // start the codegen process
    // we use the rustc_kernel_codegen crate to generate the code
    let module = rustc_kernel_codegen::base::module_codegen_hack(tcx, cgu);

    module
    //todo!()
}