use std::borrow::BorrowMut;

use rustc_codegen_ssa::traits::{MiscMethods, PreDefineMethods, TypeMembershipMethods};
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::ty::{layout::HasTyCtxt, Ty};

use crate::{function::FunctionNVVM, value::{Val, ValueNVVM}};

use super::CodegenCx;

impl<'tcx> MiscMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn vtables(
        &self,
    ) -> &std::cell::RefCell<FxHashMap<(Ty<'tcx>, Option<rustc_middle::ty::PolyExistentialTraitRef<'tcx>>), Self::Value>> {
        todo!()
    }

    fn check_overflow(&self) -> bool {
        //todo!()
        false
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
        //should return @function_name
        let module = unsafe { &mut *self.module.get() };
        //println!("get_fn_addr: {:?}", instance);
        match module.defrefs.get(&instance.def_id()) {
            Some(val) => *val,
            None => {
                // maybe an extern call
                // in that case we need to generate a call to the extern function
                generate_extern_decl(self, instance)
            }
        }
    }

    fn eh_personality(&self) -> Self::Value {
        todo!()
    }

    fn sess(&self) -> &rustc_session::Session {
        self.tcx.sess
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

fn generate_extern_decl<'m, 'tcx>(cx: &CodegenCx<'m, 'tcx>, instance: rustc_middle::ty::Instance<'tcx>) -> Val<'m> {
    let module = cx.get_module_mut();
    let name = cx.tcx.symbol_name(instance).name;

    // get the type of the function return value
    /*let abi = match self.tcx.fn_abi_of_instance(pea) {
        Ok(abi) => abi,
        Err(e) => {
            // KURVA
            todo!()
        },
    };
    
    let ty = cx.backend_type(abi.ret.layout);
    

    // create the function address
    let funcaddr = ValueNVVM::FnRef(name.to_string());
    module.create_val(funcaddr, Some(ty))*/

    cx.predefine_fn(instance, 
        rustc_middle::mir::mono::Linkage::Common, 
        rustc_middle::mir::mono::Visibility::Default, 
        name);
    let val = module.defrefs.get(&instance.def_id()).unwrap();
    *val
}