use rustc_codegen_ssa::traits::StaticMethods;

use super::CodegenCx;

impl StaticMethods for CodegenCx<'_, '_> {
    fn static_addr_of(&self, cv: Self::Value, align: rustc_target::abi::Align, kind: Option<&str>) -> Self::Value {
        todo!()
    }

    fn codegen_static(&self, def_id: rustc_hir::def_id::DefId) {
        todo!()
    }

    fn add_used_global(&self, global: Self::Value) {
        todo!()
    }

    fn add_compiler_used_global(&self, global: Self::Value) {
        todo!()
    }
}