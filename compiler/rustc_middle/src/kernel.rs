use rustc_macros::HashStable;
use rustc_middle::mir::mono::KernelMetaData;
use rustc_middle::mir::interpret::ConstAllocation;

use crate::arena::{Arena, ArenaAllocatable};
use crate::mir::mono::CodegenUnit;

#[derive(Debug, Clone, HashStable)]
pub struct KernelCode<'tcx> {
    pub const_alloc: ConstAllocation<'tcx>,	
    pub kernel_metadata: &'tcx KernelMetaData,
    pub cgu: &'tcx CodegenUnit<'tcx>,
}

impl<'tcx> KernelCode<'tcx> {
    pub fn new(const_alloc: ConstAllocation<'tcx>, kernel_metadata: &'tcx KernelMetaData, cgu: &'tcx CodegenUnit<'tcx>) -> Self {
        Self {
            const_alloc,
            kernel_metadata,
            cgu
        }
    }
}
