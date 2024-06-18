use rustc_codegen_ssa::traits::{LayoutTypeMethods, PreDefineMethods};
use rustc_middle::ty;

use crate::{function::FunctionNVVM, ty::TyNVVM};

use super::CodegenCx;

impl<'m, 'tcx> PreDefineMethods<'tcx> for CodegenCx<'m, 'tcx>{
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
        let mut module = unsafe { &mut *self.module.get() };
        // to declare the function, we need to specify the return type and the arguments
        // and wether it is a kernel function or not
        let is_kernel = self.tcx.is_kernel(instance.def_id());
        let param_env = self.tcx.param_env(instance.def_id());
        let ty = instance.kernel_ty(self.tcx, param_env);

        let pea = param_env.and((instance, ty::List::empty()));

        let abi = match self.tcx.fn_abi_of_instance(pea) {
            Ok(abi) => abi,
            Err(e) => {
                // KURVA
                todo!()
            },
        };
        
        let args: Vec<TyNVVM<'m>> = abi.args.iter().map(|arg| {
            // lower the type to the NVVM type
            self.backend_type(arg.layout)
        }).collect();
        let ret = self.backend_type(abi.ret.layout);
        let ret = if (abi.ret.layout.is_zst()) {None} else {Some(ret)};

        let mut f = FunctionNVVM::new(symbol_name.to_string(), is_kernel, ret, args);

        // we need to add the function to the module
        module.add_function(instance.def_id(), f);
    }
}