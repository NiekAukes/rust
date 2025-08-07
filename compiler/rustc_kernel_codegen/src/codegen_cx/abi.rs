use std::{marker::Tuple, num::NonZeroUsize};
use tracing::debug;

use rustc_codegen_ssa::{
    common::TypeKind,
    traits::{BaseTypeMethods, LayoutTypeMethods, TypeMembershipMethods},
};
use rustc_middle::{
    bug,
    ty::{
        self,
        layout::{FnAbiOf, FnAbiOfHelpers, LayoutOfHelpers},
        DynKind, Ty,
    },
};
use rustc_target::abi::{
    call::{FnAbi, PassMode}, Abi, AddressSpace, Align, FieldIdx, FieldsShape, HasDataLayout, Integer, PointeeInfo, Primitive, Scalar, Size, TyAndLayout, Variants
};

use crate::{
    ty::{TyNVVM, TypeNVVM},
    value::ValueNVVM,
};

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
    ) -> <Self::FnAbiOfResult as rustc_middle::ty::layout::MaybeResult<
        &'tcx rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>,
    >>::Error {
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
    ) -> <Self::LayoutOfResult as rustc_middle::ty::layout::MaybeResult<
        rustc_middle::ty::layout::TyAndLayout<'tcx>,
    >>::Error {
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
        if layout_ty.is_1zst() {
            return self.type_void();
        }
        self.lower_layout(layout_ty)
    }

    fn cast_backend_type(&self, ty: &rustc_target::abi::call::CastTarget) -> Self::Type {
        todo!()
    }

    fn fn_decl_backend_type(
        &self,
        fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>,
    ) -> Self::Type {
        let mut args = vec![]; // = abi.args.iter().enumerate().map(|(idx, arg)| {

        if fn_abi.ret.is_indirect() {
            // add a pointer to the return type
            let ret = self.backend_type(fn_abi.ret.layout);
            let ret = self.type_pointer(ret);
            args.push(ret);
        }

        for (idx, arg) in fn_abi.args.iter().enumerate() {
            // lower the type to the NVVM type
            
            match arg.mode {
                PassMode::Ignore => continue,
                PassMode::Pair(a, b) => {
                    // add 2 arguments to the list
                    
                    //let ty1 = self.backend_type(arg.layout.field(self, 0));
                    //let ty2 = self.backend_type(arg.layout.field(self, 1));
                    let (ty1, ty2) = match find_scalarpair_types(self, arg.layout) {
                        Some(t) => t,
                        None => panic!("Expected scalar pair"),
                    };
                    println!(
                        "[Kernel] fn_decl_backend_type, arg {}: pair, types: {:?}, {:?}",
                        idx, ty1, ty2
                    );
                    args.push(ty1);
                    args.push(ty2);
                }
                PassMode::Indirect { attrs, meta_attrs, on_stack } => {
                    // add a pointer to the type to the list
                    let ty = self.backend_type(arg.layout);
                    let ty = self.type_pointer(ty);
                    args.push(ty);
                }
                PassMode::Cast { .. } => todo!(),
                PassMode::Direct(_) => {
                    // basic case, lower the type and add it to the list
                    let ty = self.backend_type(arg.layout);
                    args.push(ty);
                }
            }
        }
        let ret = if fn_abi.ret.is_indirect() {
            self.type_void()
        } else {
            self.backend_type(fn_abi.ret.layout)
        };

        // build the type
        let mut module = self.get_module_mut();
        module.ty_from_type(TypeNVVM::Fn(args, ret))
    }

    fn fn_ptr_backend_type(
        &self,
        fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>,
    ) -> Self::Type {
        let fn_ty = self.fn_decl_backend_type(fn_abi);
        let mut module = self.get_module_mut();
        module.ty_from_type(TypeNVVM::Pointer(fn_ty))
    }

    fn reg_backend_type(&self, ty: &rustc_target::abi::call::Reg) -> Self::Type {
        todo!()
    }

    fn immediate_backend_type(
        &self,
        layout: rustc_middle::ty::layout::TyAndLayout<'tcx>,
    ) -> Self::Type {
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

    fn type_kind(&self, ty: Self::Type) -> TypeKind {
        match *ty {
            TypeNVVM::I(_) => TypeKind::Integer,
            TypeNVVM::F32 => TypeKind::Float,
            TypeNVVM::F64 => TypeKind::Float,
            TypeNVVM::Pointer(_) => TypeKind::Pointer,
            TypeNVVM::Array(_, _) => TypeKind::Array,
            TypeNVVM::Fn(_, _) => TypeKind::Function,
            TypeNVVM::Struct(_) => TypeKind::Struct,
            TypeNVVM::Union(_) => TypeKind::Struct,
            TypeNVVM::AdtDefForwardDecl(_, _) => TypeKind::Struct,
            TypeNVVM::Zst => TypeKind::Struct,
        }
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
    pub fn lower_layout(&self, layout: TyAndLayout<'tcx, Ty<'tcx>>) -> TyNVVM<'m> {
        let module = unsafe { &mut *self.module.get() };
        println!("lower_layout, FieldsShape: {:?}", layout.fields);
        println!("lower_layout, abi: {:?}", layout.abi);
        println!("lower_layout, ty: {:?}", layout.ty);

        match layout.abi {
            Abi::Scalar(_) | Abi::Vector { .. } => return self.lower_ty(&layout.ty),
            Abi::ScalarPair(s1, s2) => {
                let ty1 = self.scalar_type_at(s1);
                let ty2 = self.scalar_type_at(s2);
                return self.type_struct(&[ty1, ty2], false);
            }
            Abi::Uninhabited | Abi::Aggregate { .. } => {},
        }


        match layout.fields {
            FieldsShape::Array { .. } => {
                self.lower_ty(&layout.ty)
            }
            FieldsShape::Arbitrary { ref offsets, ref memory_index } => {
                let (llfields, packed) = struct_llfields(self, layout);
                self.type_struct(&llfields, packed)
            }
            FieldsShape::Primitive | FieldsShape::Union(_) => {
                let fill = self.type_padding_filler(layout.size, layout.align.abi);
                let packed = false;
                self.type_struct(&[fill], packed)
            }
        }
    }

    /// Return an LLVM type that has at most the required alignment,
    /// and exactly the required size, as a best-effort padding array.
    pub(crate) fn type_padding_filler(&self, size: Size, align: Align) -> TyNVVM<'m> {
        let unit = Integer::approximate_align(self, align);
        let size = size.bytes();
        let unit_size = unit.size().bytes();
        assert_eq!(size % unit_size, 0);
        self.type_array(self.type_from_integer(unit), size / unit_size)
    }

    pub fn lower_ty(&self, ty: &Ty<'tcx>) -> TyNVVM<'m> {
        let module = unsafe { &mut *self.module.get() };

        // check if the type is already in the cache
        let tc = unsafe { &mut *self.typecache.get() };
        if let Some(t) = tc.get(ty) {
            return *t;
        }

        let lowered_ty = match ty.kind() {
            ty::Int(n) => {
                let bitwidth = match n.bit_width() {
                    Some(w) => w,
                    None => 64, // only isize and usize have no bit width
                };
                module.ty_from_type(crate::ty::TypeNVVM::I(bitwidth as usize))
            }
            ty::Uint(n) => {
                let bitwidth = match n.bit_width() {
                    Some(w) => w,
                    None => 64, // only isize and usize have no bit width
                };
                module.ty_from_type(crate::ty::TypeNVVM::I(bitwidth as usize))
            }
            ty::Float(n) => match n {
                rustc_middle::ty::FloatTy::F32 => module.ty_from_type(crate::ty::TypeNVVM::F32),
                rustc_middle::ty::FloatTy::F64 => module.ty_from_type(crate::ty::TypeNVVM::F64),
                _ => {
                    todo!()
                }
            },

            ty::Array(ty, len) => {
                let ty = self.lower_ty(ty);
                let len = len.eval_target_usize(self.tcx, ty::ParamEnv::reveal_all());
                module.ty_from_type(crate::ty::TypeNVVM::Array(ty, len as usize))
            }

            ty::Slice(ty) => {
                let ty = self.lower_ty(ty);
                //module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
                module.ty_from_type(crate::ty::TypeNVVM::Array(ty, 0))
            }
            ty::RawPtr(ty, _) => {
                let ty = self.lower_ty(ty);
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
            }
            ty::Adt(adtdef, gargs) if !adtdef.is_enum() => {
                // forward declare this type, we need to do this because some types
                // may be recursive
                let name = match module.declare_adt(adtdef.did()) {
                    Err(name) => name,
                    Ok(name) => {
                        let mut tys = Vec::new();
                        for field in adtdef.non_enum_variant().fields.iter() {
                            let ty = self.lower_ty(&field.ty(self.tcx, gargs));
                            tys.push(ty);
                        }
                        let ty = module.ty_from_type(crate::ty::TypeNVVM::Struct(tys));
                        module.define_adt(adtdef.did(), ty);
                        name
                    }
                };

                module.ty_from_type(TypeNVVM::AdtDefForwardDecl(adtdef.did(), name))
            }

            ty::Adt(adtdef, gargs) if adtdef.is_struct() || adtdef.is_enum() => {
                // enums are represented as a struct with an index, and a union over the different variants
                // TODO: could be optimized, right now we do ty_from_type twice for each variant
                let mut tys = Vec::new();
                let index = module.ty_from_type(crate::ty::TypeNVVM::I(8));
                for variant in adtdef.variants().iter() {
                    let mut variant_tys = Vec::new();
                    variant_tys.push(index);
                    for field in variant.fields.iter() {
                        let ty = self.lower_ty(&field.ty(self.tcx, gargs));
                        variant_tys.push(ty);
                    }
                    let struc = module.ty_from_type(crate::ty::TypeNVVM::Struct(variant_tys));
                    tys.push(struc);
                }

                // because all types in a union need to have the same size, we need to pad the
                // variants to the size of the largest variant
                let max_size = tys.iter().map(|ty| ty.size(module)).max().unwrap_or(0);
                let mut padded_tys = Vec::new();
                for ty in tys.iter() {
                    let size = ty.size(module);
                    if size < max_size {
                        let padding = module.ty_from_type(crate::ty::TypeNVVM::Array(
                            index,
                            (max_size - size) as usize,
                        ));
                        let padded = module.ty_from_type(crate::ty::TypeNVVM::Struct(vec![*ty, padding]));
                        padded_tys.push(padded);
                    } else {
                        padded_tys.push(*ty);
                    }
                }

                module.ty_from_type(crate::ty::TypeNVVM::Union(padded_tys))
            }

            ty::Adt(adtdef, gargs) if adtdef.is_box() => {
                let first = adtdef.non_enum_variant().single_field();
                let ty = self.lower_ty(&first.ty(self.tcx, gargs));
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
            }

            ty::Adt(adtdef, gargs) if adtdef.is_union() => {
                let mut tys = Vec::new();
                for field in adtdef.non_enum_variant().fields.iter() {
                    let ty = self.lower_ty(&field.ty(self.tcx, gargs));
                    tys.push(ty);
                }
                module.ty_from_type(crate::ty::TypeNVVM::Union(tys))
            }

            ty::Adt(_, _) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Ref(_, ty, _) => {
                let ty = self.lower_ty(ty);
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(ty))
            }

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
            }

            ty::Str => {
                let i8 = module.ty_from_type(crate::ty::TypeNVVM::I(8));
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(i8))
            }

            ty::Bool => module.ty_from_type(crate::ty::TypeNVVM::I(1)),

            ty::Char => module.ty_from_type(crate::ty::TypeNVVM::I(32)),

            ty::FnPtr(sig) => {
                //todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
                //this should be a function pointer
                //to the (instantiated) function signature
                //let inputs_tys = sig.skip_binder().inputs().iter().map(|ty| self.lower_ty(ty)).collect::<Vec<_>>();
                let inputs = sig.skip_binder().inputs();
                let mut inputs_tys = Vec::new();
                for ty in inputs.iter() {
                    let ty = self.lower_ty(ty);
                    inputs_tys.push(ty);
                }
                let output_ty = self.lower_ty(&sig.skip_binder().output());

                let fn_ty = module.ty_from_type(crate::ty::TypeNVVM::Fn(inputs_tys, output_ty));
                module.ty_from_type(crate::ty::TypeNVVM::Pointer(fn_ty))
            }

            ty::Foreign(did) => {
                //todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
                // oof this is quite hard, I think this should just be a ZST
                self.type_void()
            }

            ty::Pat(ty, pat) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::FnDef(did, substs) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Dynamic(bounds, region, dkind) => {
                //todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
                match dkind {
                    DynKind::Dyn => {
                        let i8 = module.ty_from_type(crate::ty::TypeNVVM::I(8));
                        module.ty_from_type(crate::ty::TypeNVVM::Pointer(i8))
                    }

                    DynKind::DynStar => {
                        todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
                    }
                }
            }

            ty::Closure(did, substs) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
                // this should be just a function pointer

                // BE CAREFUL WITH skip_binder
                // let sig = self.tcx.
                // let inputs_tys = sig.inputs().iter().map(|ty| self.lower_ty(ty)).collect::<Vec<_>>();
                // let output_ty = self.lower_ty(&sig.output());

                // let fn_ty = module.ty_from_type(crate::ty::TypeNVVM::Fn(inputs_tys, output_ty));
                // module.ty_from_type(crate::ty::TypeNVVM::Pointer(fn_ty))
                //self.type_voidptr()
            }

            ty::Coroutine(did, substs) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::CoroutineClosure(did, substs) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Never => module.ty_from_type(crate::ty::TypeNVVM::Zst),

            ty::CoroutineWitness(did, substs) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Alias(akind, aty) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Bound(dbidx, boundty) => {
                todo!("unimplemented type: {:#?} with kind: {:?}", ty, ty.kind())
                // bounds checking has already happened, so lower this to the inner type
                //boundty.kind.
            }

            ty::Param(param) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Infer(infer) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Placeholder(placeholder) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            }

            ty::Error(err) => {
                todo!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
            } // _ => {
              //     bug!("unimplemented type: {:?} with kind: {:?}", ty, ty.kind())
              // }
        };

        // put the lowered type in the cache
        tc.insert(*ty, lowered_ty);
        lowered_ty
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
        let void = self.type_i8();
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

fn struct_llfields<'m, 'tcx>(
    cx: &CodegenCx<'m, 'tcx>,
    layout: TyAndLayout<'tcx, Ty<'tcx>>,
) -> (Vec<TyNVVM<'m>>, bool) {
    debug!("struct_llfields: {:#?}", layout);
    let field_count = layout.fields.count();

    let mut packed = false;
    let mut offset = Size::ZERO;
    let mut prev_effective_align = layout.align.abi;
    let mut result: Vec<_> = Vec::with_capacity(1 + field_count * 2);
    for i in layout.fields.index_by_increasing_offset() {
        let target_offset = layout.fields.offset(i as usize);
        let field = layout.field(cx, i);
        let effective_field_align =
            layout.align.abi.min(field.align.abi).restrict_for_offset(target_offset);
        packed |= effective_field_align < field.align.abi;

        debug!(
            "struct_llfields: {}: {:?} offset: {:?} target_offset: {:?} \
                effective_field_align: {}",
            i,
            field,
            offset,
            target_offset,
            effective_field_align.bytes()
        );
        assert!(target_offset >= offset);
        let padding = target_offset - offset;
        if padding != Size::ZERO {
            // let padding_align = prev_effective_align.min(effective_field_align);
            // assert_eq!(offset.align_to(padding_align) + padding, target_offset);
            // result.push(cx.type_padding_filler(padding, padding_align));
            // debug!("    padding before: {:?}", padding);
            todo!("padding in struct_llfields")
        }
        result.push(cx.lower_layout(field));
        offset = target_offset + field.size;
        prev_effective_align = effective_field_align;
    }
    if layout.is_sized() && field_count > 0 {
        if offset > layout.size {
            bug!("layout: {:#?} stride: {:?} offset: {:?}", layout, layout.size, offset);
        }
        let padding = layout.size - offset;
        if padding != Size::ZERO {
            let padding_align = prev_effective_align;
            result.push(cx.type_padding_filler(padding, padding_align));
        }
    } else {
        debug!("struct_llfields: offset: {:?} stride: {:?}", offset, layout.size);
    }
    (result, packed)
}

pub trait Lower<'m, 'tcx> {
    fn lower(&self, cx: &CodegenCx<'m, 'tcx>) -> TyNVVM<'m>;
}

impl<'m, 'tcx> Lower<'m, 'tcx> for Scalar {
    fn lower(&self, cx: &CodegenCx<'m, 'tcx>) -> TyNVVM<'m> {
        cx.scalar_type_at(*self)
    }
}


fn find_scalarpairs<'m, 'tcx>(ty: TyNVVM<'m>, scalars: &mut Vec<TyNVVM<'m>>) {
    match *ty {
        TypeNVVM::Struct(ref tys) => {
            for ty in tys.iter() {
                find_scalarpairs(*ty, scalars);
                if scalars.len() >= 2 {
                    return;
                }
            }
        },
        TypeNVVM::F32 | TypeNVVM::F64 | TypeNVVM::I(_) 
        | TypeNVVM::Pointer(_) | TypeNVVM::Array(_, _) => {
            scalars.push(ty);
        },
        _ => bug!("find_first_scalarpair: {:?}", ty),
        
    }
}

pub fn find_scalarpair_types<'m, 'tcx>(
    cx: &CodegenCx<'m, 'tcx>,
    layout: TyAndLayout<'tcx, Ty<'tcx>>,
) -> Option<(TyNVVM<'m>, TyNVVM<'m>)> {
    match layout.abi {
        Abi::ScalarPair(a, b) => {
            let a = cx.scalar_type_at(a);
            let b = cx.scalar_type_at(b);
            //let lty = cx.lower_layout(layout);
            // let mut scalars = Vec::new();
            // find_scalarpairs(lty, &mut scalars);
            // if scalars.len() == 2 {
            //     Some((scalars[0], scalars[1]))
            // } else {
                
            //     panic!("find_scalarpair_types: {:?}, scalars found: {:?}, layout: {:#?}", lty, scalars, layout);
            // }
            Some((a, b))
        }
        _ => None,
    }
}
