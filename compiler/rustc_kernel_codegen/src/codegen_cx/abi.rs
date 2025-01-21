use std::marker::Tuple;

use rustc_codegen_ssa::traits::{BaseTypeMethods, LayoutTypeMethods, TypeMembershipMethods};
use rustc_middle::{bug, ty::{self, layout::{FnAbiOfHelpers, LayoutOfHelpers}, Ty}};
use rustc_target::abi::{call::PassMode, Abi, AddressSpace, HasDataLayout, PointeeInfo, Primitive, Scalar, Size, TyAndLayout, Variants};

use crate::{ty::{TyNVVM, TypeNVVM}, value::ValueNVVM};

use super::CodegenCx;

impl<'tcx> TypeMembershipMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn add_type_metadata(&self, _function: Self::Function, _typeid: String) {}

    fn set_type_metadata(&self, _function: Self::Function, _typeid: String) {}

    fn typeid_metadata(&self, _typeid: String) -> Option<Self::Value> {
        None
    }

    fn add_kcfi_type_metadata(&self, _function: Self::Function, _typeid: u32) {}

    fn set_kcfi_type_metadata(&self, _function: Self::Function, _typeid: u32) {}
}

impl<'tcx> FnAbiOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    type FnAbiOfResult = &'tcx rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>;

    fn handle_fn_abi_err(
        &self,
        err: rustc_middle::ty::layout::FnAbiError<'tcx>,
        span: rustc_span::Span,
        fn_abi_request: rustc_middle::ty::layout::FnAbiRequest<'tcx>,
    ) -> <Self::FnAbiOfResult as rustc_middle::ty::layout::MaybeResult<&'tcx rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>>>::Error {
        todo!()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    type LayoutOfResult = rustc_middle::ty::layout::TyAndLayout<'tcx>;

    fn handle_layout_err(
        &self,
        err: rustc_middle::ty::layout::LayoutError<'tcx>,
        span: rustc_span::Span,
        ty: Ty<'tcx>,
    ) -> <Self::LayoutOfResult as rustc_middle::ty::layout::MaybeResult<rustc_middle::ty::layout::TyAndLayout<'tcx>>>::Error {
        todo!()
    }
}

impl HasDataLayout for CodegenCx<'_, '_> {
    fn data_layout(&self) -> &rustc_target::abi::TargetDataLayout {
        &self.tcx.data_layout
    }
}

impl<'tcx> LayoutTypeMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn backend_type(&self, layout_ty: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> Self::Type {
        if layout_ty.is_zst() {
            return self.type_void();
        }
        self.lower_ty(&layout_ty.ty)
    }

    fn cast_backend_type(&self, ty: &rustc_target::abi::call::CastTarget) -> Self::Type {
        todo!()
    }

    fn fn_decl_backend_type(&self, fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        let mut args = vec![];// = abi.args.iter().enumerate().map(|(idx, arg)| {
            for (idx, arg) in fn_abi.args.iter().enumerate() {
                // lower the type to the NVVM type
                println!("Arg: {:?}", arg);
                match arg.mode {
                    PassMode::Ignore => continue,
                    PassMode::Pair(_, _) => {
                        // add 2 arguments to the list
                        let ty1 = self.backend_type(arg.layout.field(self, 0));
                        let ty2 = self.backend_type(arg.layout.field(self, 1));
                        args.push(ty1);
                        args.push(ty2);
                    }
                    PassMode::Indirect { .. } => todo!(),
                    PassMode::Cast { .. } => todo!(),
                    PassMode::Direct(_) => {
                        // basic case, lower the type and add it to the list
                        let ty = self.backend_type(arg.layout);
                        args.push(ty);
                    }
                }
                
            }
            let ret = self.backend_type(fn_abi.ret.layout);
    
            // build the type
            let mut module = self.get_module_mut();
            module.ty_from_type(TypeNVVM::Fn(args, ret))
    }

    fn fn_ptr_backend_type(&self, fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!()
    }

    fn reg_backend_type(&self, ty: &rustc_target::abi::call::Reg) -> Self::Type {
        todo!()
    }

    fn immediate_backend_type(&self, layout: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> Self::Type {
        self.backend_type(layout)
    }

    fn is_backend_immediate(&self, layout: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> bool {
        //println!("is_backend_immediate, abi: {:?}", layout.abi);
        match layout.abi {
            Abi::Scalar(_) | Abi::Vector { .. } => true,
            Abi::ScalarPair(..) | Abi::Uninhabited | Abi::Aggregate { .. } => false,
        }
    }

    fn is_backend_scalar_pair(&self, layout: rustc_middle::ty::layout::TyAndLayout<'tcx>) -> bool {
        //println!("is_backend_scalar_pair, abi: {:?}", layout.abi);
        match layout.abi {
            Abi::ScalarPair(..) => true,
            Abi::Uninhabited | Abi::Scalar(_) | Abi::Vector { .. } | Abi::Aggregate { .. } => false,
        }
    }

    fn scalar_pair_element_backend_type(
        &self,
        layout: rustc_middle::ty::layout::TyAndLayout<'tcx>,
        index: usize,
        immediate: bool,
    ) -> Self::Type {
        // This must produce the same result for `repr(transparent)` wrappers as for the inner type!
        // In other words, this should generally not look at the type at all, but only at the
        // layout.
        let Abi::ScalarPair(a, b) = layout.abi else {
            bug!("scalarpair not applicable: {:?}", layout);
        };
        let scalar = [a, b][index];

        // Make sure to return the same type `immediate_llvm_type` would when
        // dealing with an immediate pair. This means that `(bool, bool)` is
        // effectively represented as `{i8, i8}` in memory and two `i1`s as an
        // immediate, just like `bool` is typically `i8` in memory and only `i1`
        // when immediate. We need to load/store `bool` as `i8` to avoid
        // crippling LLVM optimizations or triggering other LLVM bugs with `i1`.
        if immediate && scalar.is_bool() {
            return self.type_i1();
        }

        self.scalar_type_at(scalar)
    }
}

impl<'tcx> BaseTypeMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn type_i1(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::I(1))
    }

    fn type_i8(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::I(8))
    }

    fn type_i16(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::I(16))
    }

    fn type_i32(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::I(32))
    }

    fn type_i64(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::I(64))
    }

    fn type_i128(&self) -> Self::Type {
        todo!()
    }

    fn type_isize(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::I(32))
    }

    fn type_f16(&self) -> Self::Type {
        todo!()
    }

    fn type_f32(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::F32)
    }

    fn type_f64(&self) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::F64)
    }

    fn type_f128(&self) -> Self::Type {
        todo!()
    }

    fn type_array(&self, ty: Self::Type, len: u64) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        let typ = TypeNVVM::Array(ty, len as usize);
        module.ty_from_type(typ)
    }

    fn type_func(&self, args: &[Self::Type], ret: Self::Type) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        let typ = TypeNVVM::Fn(Vec::from(args), ret);
        module.ty_from_type(typ)
    }

    fn type_struct(&self, els: &[Self::Type], packed: bool) -> Self::Type {
        let mut module = unsafe { &mut *self.module.get() };
        let typ = TypeNVVM::Struct(Vec::from(els));
        module.ty_from_type(typ)
    }

    fn type_kind(&self, ty: Self::Type) -> rustc_codegen_ssa::common::TypeKind {
        todo!()
    }

    fn type_ptr(&self) -> Self::Type {
        self.type_ptr_ext(AddressSpace::DATA)
    }

    fn type_ptr_ext(&self, address_space: AddressSpace) -> Self::Type {
        if (address_space != AddressSpace::DATA) {
            println!("type_ptr_ext, address_space: {:?}", address_space);
        }
        let mut module = unsafe { &mut *self.module.get() };
        let i8 = module.ty_from_type(crate::ty::TypeNVVM::I(8));
        module.ty_from_type(crate::ty::TypeNVVM::Pointer(i8))
    }

    fn element_type(&self, ty: Self::Type) -> Self::Type {
        todo!()
    }

    fn vector_length(&self, ty: Self::Type) -> usize {
        todo!()
    }

    fn float_width(&self, ty: Self::Type) -> usize {
        todo!()
    }

    fn int_width(&self, ty: Self::Type) -> u64 {
        32
    }

    fn val_ty(&self, v: Self::Value) -> Self::Type {
        match self.get_module().valtypes.get(&v) {
            Some(ty) => *ty,
            None => {
                bug!("val_ty, value not found in valtypes: {:?}", v)
            }
        }
    }
}



impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    pub fn lower_ty(&self, ty: &Ty<'tcx>) -> TyNVVM<'m> {
        let module = unsafe { &mut *self.module.get() };
        match ty.kind() {
            ty::Int(n) => {
                let bitwidth = match n.bit_width() {
                    Some(w) => w,
                    None => 32, // only isize and usize have no bit width
                };
                module.ty_from_type(crate::ty::TypeNVVM::I(bitwidth as usize))
            }
            ty::Uint(n) => {
                let bitwidth = match n.bit_width() {
                    Some(w) => w,
                    None => 32, // only isize and usize have no bit width
                };
                module.ty_from_type(crate::ty::TypeNVVM::I(bitwidth as usize))
            },
            ty::Float(n) => {
                match n {
                    rustc_middle::ty::FloatTy::F32 => {
                        module.ty_from_type(crate::ty::TypeNVVM::F32)
                    },
                    rustc_middle::ty::FloatTy::F64 => {
                        module.ty_from_type(crate::ty::TypeNVVM::F64)
                    },
                    _ => {
                        todo!()
                    }
                }
            },

            ty::Array(ty, len) => {
                let ty = self.lower_ty(ty);
                let len = len.eval_target_usize(self.tcx, ty::ParamEnv::reveal_all());
                module.ty_from_type(crate::ty::TypeNVVM::Array(ty, len as usize))
            },

            ty::Slice(ty) => {
                let ty = self.lower_ty(ty);
                //module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
                module.ty_from_type(crate::ty::TypeNVVM::Array(ty, 0))
            },
            ty::RawPtr(ty, _) => {
                let ty = self.lower_ty(ty);
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
            },
            ty::Adt(adtdef, gargs) if !adtdef.is_enum() => {
                let mut tys = Vec::new();
                for field in adtdef.non_enum_variant().fields.iter() {
                    let ty = self.lower_ty(&field.ty(self.tcx, gargs));
                    tys.push(ty);
                }
                module.ty_from_type(crate::ty::TypeNVVM::Struct(tys))
            },

            ty::Ref(_, ty, _) => {
                let ty = self.lower_ty(ty);
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
            },

            ty::Tuple(tys) => {
                let mut tylist = Vec::new();
                for ty in tys.iter() {
                    let ty = self.lower_ty(&ty);
                    tylist.push(ty);
                }
                if tylist.len() == 0 {
                    module.ty_from_type(crate::ty::TypeNVVM::Zst)
                } else {
                    module.ty_from_type(crate::ty::TypeNVVM::Struct(tylist))
                }
            },


            ty::Str => {
                let i8 = module.ty_from_type(crate::ty::TypeNVVM::I(8));
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(i8))
            },

            _ => {
                bug!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }
        }
    }

    pub fn type_void(&self) -> TyNVVM<'m> {
        let module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::Zst)
    }

    pub fn type_pointer(&self, ty: TyNVVM<'m>) -> TyNVVM<'m> {
        let module = unsafe { &mut *self.module.get() };
        module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
    }

    pub fn type_voidptr(&self) -> TyNVVM<'m> {
        let module = unsafe { &mut *self.module.get() };
        let void = self.type_void();
        module.ty_from_type(crate::ty::TypeNVVM::Pointer(void))
    }

    pub fn scalar_type_at(&self, scalar: Scalar) -> TyNVVM<'m> {
        match scalar.primitive() {
            Primitive::Int(i, _) => self.type_from_integer(i),
            Primitive::F16 => self.type_f16(),
            Primitive::F32 => self.type_f32(),
            Primitive::F64 => self.type_f64(),
            Primitive::F128 => self.type_f128(),
            Primitive::Pointer(address_space) => self.type_ptr_ext(address_space),
        }
    }

    pub fn type_from_integer(&self, int: rustc_target::abi::Integer) -> TyNVVM<'m> {
        match int {
            rustc_target::abi::Integer::I8 => self.type_i8(),
            rustc_target::abi::Integer::I16 => self.type_i16(),
            rustc_target::abi::Integer::I32 => self.type_i32(),
            rustc_target::abi::Integer::I64 => self.type_i64(),
            rustc_target::abi::Integer::I128 => self.type_i128(),
        }
    }
}

pub trait LayoutExt {
    fn is_immediate(&self) -> bool;
}

impl<'tcx> LayoutExt for TyAndLayout<'tcx, Ty<'tcx>> {
    fn is_immediate(&self) -> bool {
        match self.abi {
            Abi::Scalar(_) | Abi::Vector { .. } => true,
            Abi::ScalarPair(..) | Abi::Uninhabited | Abi::Aggregate { .. } => false,
        }
    }
}
