use rustc_middle::mir::mono::KernelMetaData;
use rustc_middle::ty::{AdtDef, AdtKind, Const as TyConst, ConstData, ConstKind, GenericArg, GenericArgs, TyCtxt, TyKind, ValTree, VariantDef};
use rustc_middle::mir::{Const, Body};


fn literal_const_builder<'tcx>(tcx: TyCtxt<'tcx>, code: &[u8]) -> Const<'tcx> {
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
/*
// TODO! kernel builder. needs THIR or HIR
fn kernel_const_builder<'tcx>(tcx: TyCtxt<'tcx>, instance: &'tcx Instance, kernel_metadata: &'tcx KernelMetaData) -> Body<'tcx> {
    // create a kernel kind with the following definition:
    // struct Kernel<Dim, Args, Ret> where Args: Tuple { ... }
    
    let ty = tcx.type_of(kernel_metadata.entry_def_id);
    

    ty.
    todo!()
}
 */

pub fn embed_kernel<'tcx>(
    tcx: TyCtxt<'tcx>, 
    code: &[u8], 
    kernel_metadata: &KernelMetaData<'tcx>) 
    -> Body<'tcx>
 {
    let lc = literal_const_builder(tcx, code);
    //let _ = kernel_const_builder(tcx, kernel_metadata);
    // to amend the MIR such that we add an object with the code

    // as a quick fix, bind the Constant to the kernel def_id
    let did = kernel_metadata.entry_def_id;
    todo!()
}