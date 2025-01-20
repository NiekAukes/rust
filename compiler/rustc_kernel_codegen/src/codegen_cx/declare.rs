use rustc_codegen_ssa::traits::{LayoutTypeMethods, PreDefineMethods};
use rustc_middle::ty::{self, Ty};
use rustc_target::abi::call::{ArgAbi, PassMode};

use crate::{function::FunctionNVVM, ty::{TyNVVM, TypeNVVM}, value::{Val, ValueNVVM}};

use super::CodegenCx;

impl<'m, 'tcx> PreDefineMethods<'tcx> for CodegenCx<'m, 'tcx>{
    fn predefine_static(
        &self,
        def_id: rustc_hir::def_id::DefId,
        linkage: rustc_middle::mir::mono::Linkage,
        visibility: rustc_middle::mir::mono::Visibility,
        symbol_name: &str,
    ) {
        todo!()
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
        let ty = instance.ty(self.tcx, param_env);

        let pea = param_env.and((instance, ty::List::empty()));

        let abi = match self.tcx.fn_abi_of_instance(pea) {
            Ok(abi) => abi,
            Err(e) => {
                // KURVA
                todo!()
            },
        };
        
        let mut args = vec![];// = abi.args.iter().enumerate().map(|(idx, arg)| {
        for (idx, arg) in abi.args.iter().enumerate() {
            // lower the type to the NVVM type
            //println!("Arg: {:?}", arg);
            match arg.mode {
                PassMode::Ignore => continue,
                PassMode::Pair(_, _) => {
                    let ty1 = self.backend_type(arg.layout.field(self, 0));
                    let ty2 = self.backend_type(arg.layout.field(self, 1));
                    let value1 = ValueNVVM::Param {func_name: symbol_name.to_string(), idx, ty: ty1};
                    let value2 = ValueNVVM::Param {func_name: symbol_name.to_string(), idx: idx + 1, ty: ty2};
                    let val1 = module.create_val(value1, Some(ty1));
                    let val2 = module.create_val(value2, Some(ty2));
                    args.push(val1);
                    args.push(val2);
                }
                PassMode::Indirect { .. } => todo!(),
                PassMode::Cast { pad_i32, ref cast } => {
                    println!("Cast: {:?}", cast.clone());
                    println!("Pad: {:?}", pad_i32);
                    todo!()
                }
                PassMode::Direct(_) => {
                    // basic case, lower the type and add it to the list
                    let ty = self.backend_type(arg.layout);
                    let value = ValueNVVM::Param {func_name: symbol_name.to_string(), idx, ty};
                    let val = module.create_val(value, Some(ty));
                    args.push(val);
                }
            }
            
        }
        let ret = self.backend_type(abi.ret.layout);

        // build the type
        let ty = module.ty_from_type(TypeNVVM::Fn(args.iter().map(|a| {
            // all values should be params
            if let ValueNVVM::Param { ty, .. } = a.0 {
                *ty
            } else {
                panic!("Function arguments should be params");
            }
        }).collect(), ret));

        let mut f = FunctionNVVM::new(symbol_name.to_string(), is_kernel, ret, args, ty);

        // we need to add the function to the module
        module.add_function(instance.def_id(), f);
        let val = module.create_val(ValueNVVM::FnRef(symbol_name.to_string()), Some(ty));
        module.defrefs.insert(instance.def_id(), val);

        // if the function is a kernel, add a kernel interface
    }
}

impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    fn define_kernel_interface(
        &self,
        abi_args: Box<[ArgAbi<'tcx, Ty<'tcx>>]>,
        fnref: Val<'m>,
        fndef: &FunctionNVVM<'m>,
        symbol_name: &str,
    ) -> FunctionNVVM<'m>{
        // define an interface that casts the arguments to the correct types
        // and calls the function
        let mut module = unsafe { &mut *self.module.get() };
        let mut args = vec![];
        for (idx, arg) in abi_args.iter().enumerate() {
            match arg.mode {
                PassMode::Ignore => continue,
                PassMode::Pair(_, _) => {
                    // the actual function has 2 arguments for this one
                    // 
                    todo!()
                }
                PassMode::Indirect { .. } => todo!(),
                PassMode::Cast { pad_i32, ref cast } => {
                    println!("Cast: {:?}", cast.clone());
                    println!("Pad: {:?}", pad_i32);
                    todo!()
                }
                PassMode::Direct(_) => {
                    // basic case, lower the type and add it to the list
                    let ty = self.backend_type(arg.layout);
                    let value = ValueNVVM::Param {func_name: symbol_name.to_string(), idx, ty};
                    let val = module.create_val(value, Some(ty));
                    args.push(val);
                }
            }
        }
        todo!();
    }
}