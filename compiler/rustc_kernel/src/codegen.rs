use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

pub fn generate<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> (String, Vec<u8>) {
    // we need to generate the code for the kernel
    // to do that, we need to generate a cgu for the kernel
    let cgu = tcx.collect_mono_items_for_def(def_id);

    // start the codegen process
    // we use the rustc_kernel_codegen crate to generate the code
    let (name, module) = rustc_kernel_codegen::base::module_codegen(tcx, cgu);

    let mut mod_u8 = Vec::new();
    for &byte in module.iter() {
        mod_u8.push(byte as u8);
        mod_u8.push((byte >> 8) as u8);
        mod_u8.push((byte >> 16) as u8);
        mod_u8.push((byte >> 24) as u8);
    }
    (name, mod_u8)
    //todo!()
}