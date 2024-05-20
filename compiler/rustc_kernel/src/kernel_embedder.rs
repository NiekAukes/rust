use std::rc::Rc;
use std::sync::Arc;

use rustc_ast::LitKind;
use rustc_middle::mir::interpret::{Allocation, ConstAllocation};
use rustc_middle::mir::mono::{KernelMetaData, MonoItem};
use rustc_middle::thir::{Expr, ExprKind};
use rustc_middle::ty::{AdtDef, AdtKind, Const as TyConst, ConstData, ConstKind, GenericArg, GenericArgs, TyCtxt, TyKind, ValTree, VariantDef};
use rustc_middle::mir::{BasicBlock, Body, Const, ConstAlloc, Operand, Rvalue, START_BLOCK};
use rustc_mir_build::build::construct_literal_const;


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

// TODO! kernel builder
fn kernel_const_builder<'tcx>(tcx: TyCtxt<'tcx>, kernel_metadata: &'tcx KernelMetaData) -> Body<'tcx> {
    // create a kernel kind with the following definition:
    // struct Kernel<Dim, Args, Ret> where Args: Tuple { ... }
    
    let ty = tcx.type_of(kernel_metadata.entry_def_id);
    

    todo!()
}

pub fn literal_constalloc_builder<'tcx>(tcx: TyCtxt<'tcx>, code: &[u8]) -> ConstAllocation<'tcx> {
    let alloc = Allocation::from_bytes_byte_aligned_immutable(code);
    tcx.mk_const_alloc(alloc)
}

pub fn embed_kernel<'tcx>(
    tcx: TyCtxt<'tcx>, 
    code: &[u8], 
    _kernel_metadata: &KernelMetaData) 
    -> Body<'tcx>
 {
    let body = construct_literal_const(tcx, code);
    
    // pretty print the body
    println!("{:?}", body);
    //todo!();
    body
}