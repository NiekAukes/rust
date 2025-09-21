use rustc_hir::def_id::DefId;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;
use rustc_mir_build::builder::construct_literal_const;

pub fn embed_kernel<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId, name: &str, code: &[u8]) -> Body<'tcx> {
    let body = construct_literal_const(tcx, def_id, name, code);

    // pretty print the body
    //println!("{:?}", body);
    //todo!();
    body
}
