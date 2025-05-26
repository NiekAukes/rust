use rustc_codegen_ssa::traits::StaticMethods;

use crate::{value::{Const, ValueNVVM}, GlobalNVVM};

use super::CodegenCx;

impl StaticMethods for CodegenCx<'_, '_> {
    fn static_addr_of(
        &self,
        cv: Self::Value,
        align: rustc_target::abi::Align,
        kind: Option<&str>,
    ) -> Self::Value {
        // if let Some(&global_addr) = self.const_globals.borrow().get(&cv) {
        //     return global_addr;
        // }

        let module = self.get_module_mut();
        
        let cv_type = module.valtypes.get(&cv).copied()
            .expect("Constant value should have a type");

        let global_alloc = GlobalNVVM {
            name: String::new(),
            ty: Some(cv_type),
            val: None,
        };

        let global_addr = module.add_allocation(global_alloc);
        // self.const_globals.borrow_mut().insert(cv, global_addr);

        global_addr
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
