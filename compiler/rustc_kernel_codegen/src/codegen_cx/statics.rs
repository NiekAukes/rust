use rustc_abi::Align;
use rustc_codegen_ssa::traits::StaticCodegenMethods;

use super::CodegenCx;
use super::declare::fix_ptx_name;
use crate::global::GlobalNVVM;
use crate::value::{Const, Val, ValueNVVM};

impl<'m> StaticCodegenMethods for CodegenCx<'m, '_> {
    fn static_addr_of(&self, cv: Val<'m>, align: Align, kind: Option<&str>) -> Val<'m> {
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
        let ty = *module.valtypes.get(&cv).expect("StaticMethods static_addr_of must have a type");
        let ptr_ty = self.type_pointer(ty);

        println!("[Kernel Const] Creating static: {} with value {:?} and type: {:?}", name, cv, ty);

        let global = ValueNVVM::Global(GlobalNVVM { val: cv, name });

        // create a value
        let val = module.create_val(global, Some(ptr_ty));

        self.globals.borrow_mut().insert(cv, val);
        module.add_global(val);
        val
    }

    fn codegen_static(&mut self, def_id: rustc_hir::def_id::DefId) {
        todo!()
    }
}

impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    fn get_next_static_id(&self) -> usize {
        let next_id = self.next_static_id.get();
        self.next_static_id.set(next_id + 1);
        next_id
    }

    pub fn default_static_addr_of(&self, cv: Val<'m>, kind: Option<&str>) -> Val<'m> {
        self.static_addr_of(cv, Align::from_bytes(8).unwrap(), kind)
    }

    // pub fn add_expr_as_global(&self, val: Val<'m>, kind: Option<&str>) -> Val<'m> {
    //     let name = match kind {
    //         Some(k) => format!("{}_{}", k, self.get_next_static_id()),
    //         None => format!("static_{}", self.get_next_static_id()),
    //     };
    //     let global = ValueNVVM::Global(GlobalNVVM { val, name });
    //     let module = self.get_module_mut();

    //     let ty = *module.valtypes.get(&val).expect("add_as_global must have a type");
    //     let global_val = module.create_val(global, Some(self.type_pointer(ty)));
    //     self.globals.borrow_mut().insert(val, global_val);
    //     module.add_global(global_val);
    //     global_val
    // }
}
