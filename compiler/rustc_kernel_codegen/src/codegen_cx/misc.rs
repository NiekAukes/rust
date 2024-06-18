use std::borrow::BorrowMut;

use rustc_codegen_ssa::traits::{MiscMethods, TypeMembershipMethods};
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::ty::{layout::HasTyCtxt, Ty};

use crate::function::FunctionNVVM;

use super::CodegenCx;

impl<'tcx> MiscMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn vtables(
        &self,
    ) -> &std::cell::RefCell<FxHashMap<(Ty<'tcx>, Option<rustc_middle::ty::PolyExistentialTraitRef<'tcx>>), Self::Value>> {
        todo!()
    }

    fn check_overflow(&self) -> bool {
        todo!()
    }

    fn get_fn(&self, instance: rustc_middle::ty::Instance<'tcx>) -> Self::Function {
        let module = unsafe { &mut *self.module.get() };
        // very first thing to do: check if this is a kernel function
        //if (self.tcx().is_kernel(instance.def_id())) {
            // pass the instance to the kernel fn generator
        //}
        module.functions.get(&instance.def_id()).unwrap()
    }

    fn get_fn_addr(&self, instance: rustc_middle::ty::Instance<'tcx>) -> Self::Value {
        todo!()
    }

    fn eh_personality(&self) -> Self::Value {
        todo!()
    }

    fn sess(&self) -> &rustc_session::Session {
        todo!()
    }

    fn codegen_unit(&self) -> &'tcx rustc_middle::mir::mono::CodegenUnit<'tcx> {
        todo!()
    }

    fn set_frame_pointer_type(&self, llfn: Self::Function) {
        todo!()
    }

    fn apply_target_cpu_attr(&self, llfn: Self::Function) {
        todo!()
    }

    fn declare_c_main(&self, fn_type: Self::Type) -> Option<Self::Function> {
        todo!()
    }
}