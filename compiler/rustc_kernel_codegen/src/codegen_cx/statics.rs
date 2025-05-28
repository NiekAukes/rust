use rustc_codegen_ssa::traits::StaticMethods;

use crate::{
     global::GlobalNVVM, value::{Const, Val, ValueNVVM}
};

use super::{declare::fix_ptx_name, CodegenCx};

impl<'m> StaticMethods for CodegenCx<'m, '_> {
    fn static_addr_of(
        &self,
        cv: Self::Value,
        align: rustc_target::abi::Align,
        kind: Option<&str>,
    ) -> Val<'m> {
        // lookup the constant value in the globals map
        // if it exists, return the existing global variable
        // if it does not exist, create a new global variable
        if let Some(&global) = self.globals.borrow().get(&cv) {
            return global;
        }

        // create a new global variable for the static
        let name = match kind {
            Some(k) => format!("{}_{}", k, self.get_next_static_id()),
            None => format!("static_{}", self.get_next_static_id()),
        };

        let module = self.get_module_mut();
        let global = ValueNVVM::Global(GlobalNVVM { val: cv, name });

        let ty = *module.valtypes.get(&cv).expect("StaticMethods static_addr_of must have a type");
        let ptr_ty = self.type_pointer(ty);

        // create a value
        let val = module.create_val(global, Some(ptr_ty));

        self.globals.borrow_mut().insert(cv, val);
        module.add_global(val);
        val
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

impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    fn get_next_static_id(&self) -> usize {
        let next_id = self.next_static_id.get();
        self.next_static_id.set(next_id + 1);
        next_id
    }
}

