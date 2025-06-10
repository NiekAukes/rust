use super::CodegenCx;
use rustc_ast::expand::allocator::ALLOCATOR_METHODS;
use rustc_hir::lang_items::LangItem;
use rustc_span::def_id::LOCAL_CRATE;

impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    pub fn build_kernel_allocator_shims(&self) {
        let Some(allocator_did) = self.tcx.kernel_allocator(LOCAL_CRATE) else {
            return;
        };

        println!(
            "[Kernel Allocator] Found provider: {}",
            self.tcx.def_path_str(allocator_did)
        );

        // todo!("Implement kernel allocator shims");
    }
}