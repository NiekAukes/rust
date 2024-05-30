use rustc_codegen_ssa::traits::PreDefineMethods;

use super::CodegenCx;

impl<'tcx> PreDefineMethods<'tcx> for CodegenCx<'_, 'tcx>{
    fn predefine_static(
        &self,
        def_id: rustc_hir::def_id::DefId,
        linkage: rustc_middle::mir::mono::Linkage,
        visibility: rustc_middle::mir::mono::Visibility,
        symbol_name: &str,
    ) {
        //todo!()
        // we can't do stuff here because we can't modify the codegen_cx
    }

    fn predefine_fn(
        &self,
        instance: rustc_middle::ty::Instance<'tcx>,
        linkage: rustc_middle::mir::mono::Linkage,
        visibility: rustc_middle::mir::mono::Visibility,
        symbol_name: &str,
    ) {
        // because of how the function is defined,
        // we need to unsafely get the codegen_cx as mut
        //let mut cx = unsafe { &mut *(self as *const _ as *mut CodegenCx) };
    }
}