use rustc_middle::ty::{Const as TyConst, ConstData, ConstKind, TyCtxt, TyKind, ValTree};
use rustc_middle::mir::Const;

fn literal_place_builder<'tcx>(tcx: TyCtxt<'tcx>, code: &[u8]) -> Const<'tcx> {
    // we need to create a new literal place builder
    // that will be used to create the new literal place
    // that will hold the code
    
    let tykind =  TyKind::Array(tcx.types.u8, TyConst::from_target_usize(tcx, code.len() as u64));
    let ty = tcx.mk_ty_from_kind(tykind);
    let valtree = ValTree::from_raw_bytes(tcx, code);
    let const_kind = ConstKind::Value(valtree);

    let x = tcx.mk_ct_from_kind(const_kind, ty);

    Const::from_ty_const(x, tcx)
}


pub fn embed_kernel(tcx: TyCtxt<'_>, code: &[u8]) {
    let arena = tcx.arena;
    let _ = literal_place_builder(tcx, code);
    // to amend the MIR such that we add an object with the code
    // we need to add 2 new places:
    // 1 for the code literal itself
    // 1 for the object that holds a reference to the code literal
    // we also need 2 RValues that correspond with those definitions
    todo!()
}