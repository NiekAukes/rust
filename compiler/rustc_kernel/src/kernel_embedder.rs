use rustc_middle::ty::{ConstData, TyCtxt, TyKind};
use rustc_middle::mir::Place;

fn literal_place_builder<'tcx>(tcx: TyCtxt<'tcx>, code: &[u8]) -> Place<'tcx> {
    // we need to create a new literal place builder
    // that will be used to create the new literal place
    // that will hold the code
    
    let tykind =  TyKind::Array(tcx.types.u8, code.len() as u64);
    let const_kind = ConstKind::Value(ConstValue::Slice(code));
    /*let x = ConstData {
        ty: tcx.types.
        val: ConstValue::Slice(code),
        has_value: true,
    }*/
    todo!()
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