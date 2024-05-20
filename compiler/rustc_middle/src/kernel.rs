use rustc_macros::HashStable;
use rustc_middle::mir::mono::KernelMetaData;
use rustc_middle::mir::Body;

use crate::arena::{Arena, ArenaAllocatable};
use crate::mir::mono::CodegenUnit;

#[derive(Debug, Clone, HashStable)]
pub struct KernelCode<'tcx> {
    pub const_body: Body<'tcx>,
    pub kernel_metadata: &'tcx KernelMetaData,
    pub cgu: &'tcx CodegenUnit<'tcx>,
}