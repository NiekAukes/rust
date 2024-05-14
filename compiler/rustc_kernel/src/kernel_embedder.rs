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

// TODO! kernel builder. needs THIR or HIR
/*fn kernel_const_builder<'tcx>(tcx: TyCtxt<'tcx>, kernel_metadata: KernelMetaData) -> Const<'tcx> {
    // create a kernel kind with the following definition:
    // struct Kernel<Dim, Args, Ret> where Args: Tuple { ... }
    
    // we first need the kernel dimension
    let dim_ty = kernel_metadata.dim;

    // we then need the kernel arguments
    let args_ty = kernel_metadata.kernel_args;

    // we then need the kernel return type
    let ret_ty = kernel_metadata.kernel_ret; 
    
    let d = GenericArg::from(dim_ty);
    let a = GenericArg::from(args_ty);
    let r = GenericArg::from(ret_ty);

    // create the generic arguments
    let adt_def = tcx.adt_def(kernel_metadata.kernel_adt_id);
    let ty_kind = TyKind::Adt(adt_def, &generic_args);
}*/


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