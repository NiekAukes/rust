use rustc_codegen_ssa::traits::{BaseTypeMethods, LayoutTypeMethods, PreDefineMethods};
use rustc_middle::ty::{self, Ty};
use rustc_target::abi::{
    call::{ArgAbi, ArgAttribute, CastTarget, PassMode, Reg, RegKind},
    Abi, Scalar, Size,
};

use crate::{
    function::FunctionNVVM,
    ty::{TyNVVM, TypeNVVM},
    value::{Const, Val, ValueNVVM},
};

use super::{
    abi::{find_scalarpair_types, Lower},
    CodegenCx,
};

impl<'m, 'tcx> PreDefineMethods<'tcx> for CodegenCx<'m, 'tcx> {
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
            }
        };

        // fix the symbol name for PTX
        let symbol_name = fix_ptx_name(symbol_name);

        let mut arg_count = 0;

        let mut args = vec![]; // = abi.args.iter().enumerate().map(|(idx, arg)| {

        // if the return type is indirect, we need to add a pointer to the return type
        // as argument
        if abi.ret.is_indirect() {
            let ty = self.backend_type(abi.ret.layout);
            let ty = self.type_pointer(ty);
            let value = ValueNVVM::Param { func_name: symbol_name.to_string(), idx: arg_count, ty };
            let val = module.create_val(value, Some(ty));
            args.push(val);
            arg_count += 1;
        }

        for (idx, arg) in abi.args.iter().enumerate() {
            // lower the type to the NVVM type
            match arg.mode {
                PassMode::Ignore => continue,
                PassMode::Pair(a, b) => {
                    // let ty1 = self.backend_type(arg.layout.field(self, 0));
                    // let value1 = ValueNVVM::Param {func_name: symbol_name.to_string(), idx: arg_count, ty: ty1};
                    // module.create_val(value1, Some(ty1))
                    
                    let (val1, val2) = match arg.layout.abi {
                        Abi::ScalarPair(s1, s2) => {
                            let (ty1, ty2) = match find_scalarpair_types(self, arg.layout) {
                                Some(t) => t,
                                None => panic!("Expected scalar pair"),
                            };
                            let value1 = ValueNVVM::Param {
                                func_name: symbol_name.to_string(),
                                idx: arg_count,
                                ty: ty1,
                            };
                            let value2 = ValueNVVM::Param {
                                func_name: symbol_name.to_string(),
                                idx: arg_count + 1,
                                ty: ty2,
                            };
                            (
                                module.create_val(value1, Some(ty1)),
                                module.create_val(value2, Some(ty2)),
                            )
                        }
                        _ => panic!("Expected scalar pair"),
                    };

                    args.push(val1);
                    args.push(val2);
                    arg_count += 2;
                }
                PassMode::Indirect { attrs, meta_attrs, on_stack } => {
                    // we need to pass a pointer to the value
                    // CHECK: is this correct?
                    let ty = self.backend_type(arg.layout);
                    let ty_ptr = self.type_pointer(ty);
                    let value =
                        ValueNVVM::Param { func_name: symbol_name.to_string(), idx: arg_count, ty: ty_ptr };
                    let val = module.create_val(value, Some(ty_ptr));
                    args.push(val);
                    arg_count += 1;
                }
                PassMode::Cast { pad_i32, ref cast } => {
                    let ty = cast.nvvm_type(self);
                    let value =
                        ValueNVVM::Param { func_name: symbol_name.to_string(), idx: arg_count, ty };
                    let val = module.create_val(value, Some(ty));
                    args.push(val);
                    arg_count += 1;
                }
                PassMode::Direct(_) => {
                    // basic case, lower the type and add it to the list
                    let ty = self.backend_type(arg.layout);
                    let value =
                        ValueNVVM::Param { func_name: symbol_name.to_string(), idx: arg_count, ty };
                    let val = module.create_val(value, Some(ty));
                    args.push(val);
                    arg_count += 1;
                }
            }
        }
        let ret = if abi.ret.is_indirect() {
            self.type_void()
        } else {
            self.backend_type(abi.ret.layout)
        };

        // build the type
        let ty = module.ty_from_type(TypeNVVM::Fn(
            args.iter()
                .map(|a| {
                    // all values should be params
                    if let ValueNVVM::Param { ty, .. } = a.0 {
                        *ty
                    } else {
                        panic!("Function arguments should be params");
                    }
                })
                .collect(),
            ret,
        ));

        let mut f = FunctionNVVM::new(symbol_name.to_string(), is_kernel, ret, args, ty);

        // println!(
        //     "[Kernel] Predefining function: {} with args: {:?} and ret: {:?}",
        //     symbol_name, f.args, f.ret
        // );

        // we need to add the function to the module
        module.add_function(symbol_name.to_string(), f);
        let val =
            module.create_val(ValueNVVM::Constant(Const::FnRef(symbol_name.to_string())), Some(ty));
        module.defrefs.insert(symbol_name.to_string(), val);

        // if the function is a kernel, add a kernel interface
    }
}

trait NVVMType<'m, 'tcx> {
    fn nvvm_type(&self, cx: &CodegenCx<'m, 'tcx>) -> TyNVVM<'m>;
}

impl<'m, 'tcx> NVVMType<'m, 'tcx> for CastTarget {
    fn nvvm_type(&self, cx: &CodegenCx<'m, 'tcx>) -> TyNVVM<'m> {
        let unit_type = self.rest.unit.nvvm_type(cx);
        let rest_count = if self.rest.total == Size::ZERO {
            0
        } else {
            assert_ne!(
                self.rest.unit.size,
                Size::ZERO,
                "total size {:?} cannot be divided into units of zero size",
                self.rest.total
            );
            if self.rest.total.bytes() % self.rest.unit.size.bytes() != 0 {
                assert_eq!(self.rest.unit.kind, RegKind::Integer, "only int regs can be split");
            }
            self.rest.total.bytes().div_ceil(self.rest.unit.size.bytes())
        };

        // Simplify to a single unit or an array if there's no prefix.
        // This produces the same layout, but using a simpler type.
        if self.prefix.iter().all(|x| x.is_none()) {
            // We can't do this if is_consecutive is set and the unit would get
            // split on the target. Currently, this is only relevant for i128
            // registers.
            if rest_count == 1 && (!self.rest.is_consecutive || self.rest.unit != Reg::i128()) {
                return unit_type;
            }

            return cx.type_array(unit_type, rest_count);
        }

        // Generate a struct type with the prefix and the "rest" arguments.
        let prefix_args =
            self.prefix.iter().flat_map(|option_reg| option_reg.map(|reg| reg.nvvm_type(cx)));
        let rest_args = (0..rest_count).map(|_| unit_type);
        let args: Vec<_> = prefix_args.chain(rest_args).collect();
        cx.type_struct(&args, false)
    }
}

impl<'m, 'tcx> NVVMType<'m, 'tcx> for Reg {
    fn nvvm_type(&self, cx: &CodegenCx<'m, 'tcx>) -> TyNVVM<'m> {
        match self.kind {
            RegKind::Integer => match self.size.bytes() {
                1 => cx.type_i8(),
                4 => cx.type_i32(),
                8 => cx.type_i64(),
                _ => panic!("Unsupported integer size"),
            },
            RegKind::Float => match self.size.bytes() {
                4 => cx.type_f32(),
                8 => cx.type_f64(),
                _ => panic!("Unsupported float size"),
            },
            _ => panic!("Unsupported register kind"),
        }
    }
}

pub fn fix_ptx_name(name: &str) -> String {
    // replace all dots with underscores
    // because PTX doesn't allow dots in names
    name.replace(".", "_")
}
