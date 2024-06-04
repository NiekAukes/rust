use rustc_hir::def_id::{self, DefId};
use rustc_middle::mir::interpret::{Allocation, ConstAllocation};
use rustc_middle::ty::{AdtDef, AdtKind, Const as TyConst, ConstData, ConstKind, GenericArg, GenericArgs, TyCtxt, TyKind, ValTree, VariantDef};
use rustc_middle::mir::{BasicBlock, Body, Const, ConstAlloc, Operand, Rvalue, START_BLOCK};
use rustc_mir_build::build::construct_literal_const;

pub fn embed_kernel<'tcx>(
    tcx: TyCtxt<'tcx>, 
    def_id: DefId,
    name: &str,
    code: &[u8]) 
    -> Body<'tcx>
 {
    let body = construct_literal_const(tcx, def_id, name, code);
    
    // pretty print the body
    //println!("{:?}", body);
    //todo!();
    body
}