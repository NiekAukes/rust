use rustc_codegen_ssa::traits::StaticMethods;

use crate::value::ValueNVVM;

use super::CodegenCx;

impl StaticMethods for CodegenCx<'_, '_> {
    fn static_addr_of(
        &self,
        cv: Self::Value,
        align: rustc_target::abi::Align,
        kind: Option<&str>,
    ) -> Self::Value {
        if let Some(s) = kind { todo!() } else { cv }
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
