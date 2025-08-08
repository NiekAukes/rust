use std::ops::Range;

use rustc_codegen_ssa::traits::{BaseTypeMethods, ConstMethods, MiscMethods, StaticMethods};
use rustc_hir::def_id::DefId;
use rustc_middle::{
    bug,
    mir::interpret::{
        read_target_uint, AllocId, AllocRange, Allocation, ConstAllocation, GlobalAlloc, InitChunk,
        Pointer, Scalar as InterpScalar,
    },
};
use rustc_target::abi::{self, Align, HasDataLayout, Primitive, Scalar, Size, WrappingRange};

use crate::{
    codegen_cx::declare::fix_ptx_name,
    global::{ConstExpr, GlobalNVVM},
    ty::{TyNVVM, TypeNVVM},
    value::{Const, Instruction, Val, ValueNVVM},
};

use super::CodegenCx;

impl<'m, 'tcx> ConstMethods<'tcx> for CodegenCx<'m, 'tcx> {
    fn const_null(&self, t: Self::Type) -> Self::Value {
        match t.0 {
            TypeNVVM::I(1) => self.const_bool(false),
            TypeNVVM::I(8) => self.const_i8(0),
            TypeNVVM::I(16) => self.const_i16(0),
            TypeNVVM::I(32) => self.const_i32(0),
            TypeNVVM::I(64) => self.const_i64(0),
            TypeNVVM::I(128) => self.const_u128(0),
            TypeNVVM::F32 | TypeNVVM::F64 => self.const_real(t, 0.0),
            TypeNVVM::Pointer(_) => {
                let value = ValueNVVM::Constant(Const::NullPtr);
                self.get_module_mut().create_val(value, Some(t))
            }
            TypeNVVM::Struct(_) | TypeNVVM::Array(_, _) => {
                let value = ValueNVVM::Constant(Const::ZeroInitializer);
                self.get_module_mut().create_val(value, Some(t))
            }
            TypeNVVM::Zst => self.const_struct(&[], false),
            _ => {
                bug!("const_null called on unsupported type: {:?}", t)
            }
        }
    }

    fn const_undef(&self, t: Self::Type) -> Self::Value {
        let module = self.get_module_mut();
        let value = ValueNVVM::Constant(Const::Undef(t, t.size(module)));
        module.create_val(value, Some(t))
    }

    fn const_poison(&self, t: Self::Type) -> Self::Value {
        self.const_undef(t)
    }

    fn const_int(&self, t: Self::Type, i: i64) -> Self::Value {
        todo!()
    }

    fn const_uint(&self, t: Self::Type, i: u64) -> Self::Value {
        self.const_u64(i)
    }

    fn const_uint_big(&self, t: Self::Type, u: u128) -> Self::Value {
        match t.0 {
            TypeNVVM::I(1) => self.const_u8(u as u8),   
            TypeNVVM::I(8) => self.const_u8(u as u8),
            TypeNVVM::I(16) => self.const_i16(u as i16),
            TypeNVVM::I(32) => self.const_u32(u as u32),
            TypeNVVM::I(64) => self.const_u64(u as u64),
            TypeNVVM::I(128) => self.const_u128(u),
            _ => bug!("const_uint_big called on unsupported type: {:?}", t),
        }
    }

    fn const_bool(&self, val: bool) -> Self::Value {
        let value = ValueNVVM::Constant(Const::Bool(val));
        self.get_module_mut().create_val(value, Some(self.type_i1()))
    }

    fn const_i16(&self, i: i16) -> Self::Value {
        todo!()
    }

    fn const_i32(&self, i: i32) -> Self::Value {
        let value = ValueNVVM::Constant(Const::I32(i));
        let ty = self.type_i32();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_i8(&self, i: i8) -> Self::Value {
        let value = ValueNVVM::Constant(Const::I8(i));
        let ty = self.type_i8();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_u32(&self, i: u32) -> Self::Value {
        let value = ValueNVVM::Constant(Const::U32(i));
        let ty = self.type_i32();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_u64(&self, i: u64) -> Self::Value {
        let value = ValueNVVM::Constant(Const::U64(i));
        let ty = self.type_i64();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_u128(&self, i: u128) -> Self::Value {
        // TODO: Implement
        let value = ValueNVVM::Constant(Const::U64(i as u64));
        let ty = self.type_i64();
        self.get_module_mut().create_val(value, Some(ty))
    }

    fn const_usize(&self, i: u64) -> Self::Value {
        self.const_u64(i as u64)
    }

    fn const_u8(&self, i: u8) -> Self::Value {
        todo!()
    }

    fn const_real(&self, t: Self::Type, val: f64) -> Self::Value {
        todo!()
    }

    fn const_str(&self, s: &str) -> (Self::Value, Self::Value) {
        todo!()
    }

    fn const_struct(&self, elts: &[Self::Value], packed: bool) -> Self::Value {
        // create a struct from the elements
        // unwrap the constants first
        let consts = elts
            .iter()
            .map(|v| match v.0 {
                ValueNVVM::Constant(c) => c.clone(),
                _ => bug!("const_struct called with non-constant value: {:?}", v),
            })
            .collect::<Vec<_>>();
        let struct_const = Const::Struct(consts);
        let ty = struct_const.get_ty(self.get_module_mut());
        self.get_module_mut().create_val(ValueNVVM::Constant(struct_const), Some(ty))
    }

    fn const_to_opt_uint(&self, v: Self::Value) -> Option<u64> {
        // if the value is a constant, we can extract the value
        // otherwise we can't
        // this is for optimization purposes
        match *v {
            ValueNVVM::Constant(Const::U8(u)) => Some(u as u64),
            ValueNVVM::Constant(Const::U16(u)) => Some(u as u64),
            ValueNVVM::Constant(Const::U32(u)) => Some(u as u64),
            ValueNVVM::Constant(Const::U64(u)) => Some(u),
            ValueNVVM::Constant(Const::U128(u)) => None,
            ValueNVVM::Constant(Const::I8(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }
            ValueNVVM::Constant(Const::I16(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }
            ValueNVVM::Constant(Const::I32(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }
            ValueNVVM::Constant(Const::I64(i)) => {
                if i >= 0 {
                    Some(i as u64)
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    fn const_to_opt_u128(&self, v: Self::Value, sign_ext: bool) -> Option<u128> {
        None // Not supported
    }

    fn const_data_from_alloc(
        &self,
        alloc: rustc_middle::mir::interpret::ConstAllocation<'tcx>,
    ) -> Self::Value {
        // return the const value of what to allocate.
        let alloc = alloc.inner();
        // We expect that callers of const_alloc_to_llvm will instead directly codegen a pointer or
        // integer for any &ZST where the ZST is a constant (i.e. not a static). We should never be
        // producing empty LLVM allocations as they're just adding noise to binaries and forcing less
        // optimal codegen.

        let mut llvals = Vec::with_capacity(alloc.provenance().ptrs().len() + 1);
        let dl = self.data_layout();
        let pointer_size = dl.pointer_size.bytes() as usize;

        // Note: this function may call `inspect_with_uninit_and_ptr_outside_interpreter`,
        // so `range` must be within the bounds of `alloc` and not contain or overlap a relocation.
        fn append_chunks_of_init_and_uninit_bytes<'m, 'a>(
            llvals: &mut Vec<Val<'m>>,
            cx: &'a CodegenCx<'m, '_>,
            alloc: &'a Allocation,
            range: Range<usize>,
        ) {
            let chunks = alloc.init_mask().range_as_init_chunks(range.clone().into());

            let chunk_to_llval = move |chunk| match chunk {
                InitChunk::Init(range) => {
                    let range = (range.start.bytes() as usize)..(range.end.bytes() as usize);
                    let bytes = alloc.inspect_with_uninit_and_ptr_outside_interpreter(range);
                    cx.const_bytes(bytes)
                }
                InitChunk::Uninit(range) => {
                    let len = range.end.bytes() - range.start.bytes();
                    cx.const_undef(cx.type_array(cx.type_i8(), len))
                }
            };

            // Generating partially-uninit consts is limited to small numbers of chunks,
            // to avoid the cost of generating large complex const expressions.
            // For example, `[(u32, u8); 1024 * 1024]` contains uninit padding in each element,
            // and would result in `{ [5 x i8] zeroinitializer, [3 x i8] undef, ...repeat 1M times... }`.
            let max = cx.sess().opts.unstable_opts.uninit_const_chunk_threshold;
            let allow_uninit_chunks = chunks.clone().take(max.saturating_add(1)).count() <= max;

            if allow_uninit_chunks {
                llvals.extend(chunks.map(chunk_to_llval));
            } else {
                // If this allocation contains any uninit bytes, codegen as if it was initialized
                // (using some arbitrary value for uninit bytes).
                let bytes = alloc.inspect_with_uninit_and_ptr_outside_interpreter(range);
                llvals.push(cx.const_bytes(bytes));
            }
        }

        let mut next_offset = 0;
        for &(offset, prov) in alloc.provenance().ptrs().iter() {
            let offset = offset.bytes();
            assert_eq!(offset as usize as u64, offset);
            let offset = offset as usize;
            if offset > next_offset {
                // This `inspect` is okay since we have checked that it is not within a relocation, it
                // is within the bounds of the allocation, and it doesn't affect interpreter execution
                // (we inspect the result after interpreter execution).
                append_chunks_of_init_and_uninit_bytes(
                    &mut llvals,
                    self,
                    alloc,
                    next_offset..offset,
                );
            }
            let ptr_offset = read_target_uint(
                dl.endian,
                // This `inspect` is okay since it is within the bounds of the allocation, it doesn't
                // affect interpreter execution (we inspect the result after interpreter execution),
                // and we properly interpret the relocation as a relocation pointer offset.
                alloc.inspect_with_uninit_and_ptr_outside_interpreter(
                    offset..(offset + pointer_size),
                ),
            )
            .expect("const_alloc_to_llvm: could not read relocation pointer")
                as u64;

            let address_space = self.tcx.global_alloc(prov.alloc_id()).address_space(self);

            let llval = self.scalar_to_backend(
                InterpScalar::from_pointer(
                    Pointer::new(prov, Size::from_bytes(ptr_offset)),
                    &self.tcx,
                ),
                Scalar::Initialized {
                    value: Primitive::Pointer(address_space),
                    valid_range: WrappingRange { start: 0, end: !0 },
                },
                self.type_ptr_ext(address_space),
            );
            llvals.push(llval);
            next_offset = offset + pointer_size;
        }

        if alloc.len() >= next_offset {
            let range = next_offset..alloc.len();
            // This `inspect` is okay since we have check that it is after all relocations, it is
            // within the bounds of the allocation, and it doesn't affect interpreter execution (we
            // inspect the result after interpreter execution).
            append_chunks_of_init_and_uninit_bytes(&mut llvals, self, alloc, range);
        }

        self.const_struct(&llvals, true)
    }

    fn scalar_to_backend(
        &self,
        cv: rustc_middle::mir::interpret::Scalar,
        layout: rustc_target::abi::Scalar,
        llty: Self::Type,
    ) -> Self::Value {
        let value = match cv {
            InterpScalar::Int(i) => {
                let sz = i.size().bytes();
                match sz {
                    1 => self.const_i8(i.try_to_i8().unwrap()),
                    2 => self.const_i16(i.try_to_i16().unwrap()),
                    4 => self.const_i32(i.try_to_i32().unwrap()),
                    8 => self.const_i64(i.try_to_i64().unwrap()),
                    16 => self.const_i64(i.try_to_i128().unwrap() as i64),
                    _ => todo!(),
                }
            }
            InterpScalar::Ptr(p, _) => {
                let (prov, offset) = p.into_parts();
                match self.tcx.global_alloc(prov.alloc_id()) {
                    GlobalAlloc::Memory(alloc) => {
                        return self.const_data_from_alloc(alloc);
                    }
                    GlobalAlloc::Function(instance) => {
                        // make an fnref to the function
                        let symbol_name = self.tcx.symbol_name(instance).name.to_string();
                        let symbol_name = fix_ptx_name(&symbol_name);

                        // get the module
                        let module = self.get_module();
                        return *module.defrefs.get(&symbol_name).unwrap();
                    }
                    GlobalAlloc::Static(def_id) => {
                        let alloc = self.tcx.eval_static_initializer(def_id).unwrap();
                        return self.const_data_from_alloc(alloc);
                    }
                    GlobalAlloc::VTable(ty, binder) => {
                        let key = (ty, binder);
                        let alloc_id = self.tcx.vtable_allocation(key);
                        match self.tcx.global_alloc(alloc_id) {
                            GlobalAlloc::Memory(alloc) => {
                                let val = self.const_data_from_alloc(alloc);
                                return val;
                            }
                            _ => {
                                bug!("vtable allocation for {:?} is not a memory allocation", key);
                            }
                        }
                    }
                }
                todo!()
            }
        };

        value
    }

    fn const_bitcast(&self, val: Val<'m>, ty: TyNVVM<'m>) -> Val<'m> {
        // create a bitcast instruction
        // let expr = ConstExpr::BitCast { val, ty };
        // let value = ValueNVVM::ConstExpr(expr);
        // let val = self.get_module_mut().create_val(value, Some(ty));
        let val = self.inner_bitcast(val, ty);

        self.static_addr_of(val, Align::from_bytes(8).unwrap(), None)
    }

    fn const_ptr_byte_offset(&self, val: Val<'m>, offset: rustc_target::abi::Size) -> Val<'m> {
        let mut module = self.get_module_mut();
        // infer the type of the operation, in practice the val is almost always a pointer
        let val_ty = *module.valtypes.get(&val).expect("const_ptr_byte_offset must have a type");

        // check if the value is already a i8 pointer, otherwise bitcast it to a pointer to i8
        let is_i8 = match val_ty.0 {
            TypeNVVM::Pointer(ty) => match *ty.0 {
                TypeNVVM::Array(aty, _) => {
                    // if the pointer is an array, we need to get the element type
                    if aty == self.type_i8() {
                        // if the element type is i8, we can do a byte offset directly
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            _ => {
                // if the value is not a pointer, we can't do a pointer offset
                bug!("const_ptr_byte_offset called on non-pointer type: {:?}", val_ty);
            }
        };

        let i8ptr_ty = self.type_pointer(self.type_i8());
        let gepval = if is_i8 {
            // if the value is already a pointer to i8, we can do a byte offset directly
            val
        } else {
            // otherwise bitcast it to an i8*
            self.inner_bitcast(val, i8ptr_ty)
        };
        // create a GEP instruction to add the offset
        let gep = ConstExpr::GEP {
            ty: self.type_i8(),
            val: gepval,
            indices: vec![Const::U64(offset.bytes() as u64)],
        };
        let value = ValueNVVM::ConstExpr(gep);
        let val = module.create_val(value, Some(i8ptr_ty));

        self.static_addr_of(val, Align::from_bytes(8).unwrap(), None)
    }
}

impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    pub fn const_i64(&self, i: i64) -> Val<'m> {
        let value = ValueNVVM::Constant(Const::I64(i));
        let ty = self.type_i64();
        self.get_module_mut().create_val(value, Some(ty))
    }

    pub fn const_i32(&self, i: i32) -> Val<'m> {
        let value = ValueNVVM::Constant(Const::I32(i));
        let ty = self.type_i32();
        self.get_module_mut().create_val(value, Some(ty))
    }

    pub fn const_array(
        &self,
        elts: &[Val<'m>],
    ) -> Val<'m> {
        // create an array from the elements
        // unwrap the constants first
        let consts = elts
            .iter()
            .map(|v| match v.0 {
                ValueNVVM::Constant(c) => c.clone(),
                _ => bug!("const_array called with non-constant value: {:?}", v),
            })
            .collect::<Vec<_>>();
        let arr_const = Const::Arr(consts);
        let ty = arr_const.get_ty(self.get_module_mut());
        self.get_module_mut().create_val(ValueNVVM::Constant(arr_const), Some(ty))
    }

    pub fn const_bytes(&self, bytes: &[u8]) -> Val<'m> {
        let consts = bytes.iter().map(|x| Const::U8(*x)).collect();
        let val = ValueNVVM::Constant(Const::Arr(consts));
        let ty = self.type_array(self.type_i8(), bytes.len() as u64);
        self.get_module_mut().create_val(val, Some(ty))
    }

    pub fn inner_bitcast(&self, val: Val<'m>, ty: TyNVVM<'m>) -> Val<'m> {
        // create a bitcast instruction
        let expr = ConstExpr::BitCast { val, ty };
        let value = ValueNVVM::ConstExpr(expr);
        self.get_module_mut().create_val(value, Some(ty))
    }


    pub(crate) fn get_static(&self, def_id: DefId) -> Val<'m> {
        // let instance = Instance::mono(self.tcx, def_id);
        // trace!(?instance);

        // let Changed = 1;
        // let nested = match self.tcx.def_kind(def_id) {
        //     DefKind::Static {nested, ..} => nested,
        //     _ if self.tcx.is_kernel(def_id) => false,
        //     _ => bug!("get_static: expected a static, but got {:?}", def_id),
        // };
        // // Nested statics do not have a type, so pick a dummy type and let `codegen_static` figure out
        // // the llvm type from the actual evaluated initializer.
        // let llty = if nested {
        //     self.type_i8()
        // } else {
        //     let ty = instance.ty(self.tcx, ty::ParamEnv::reveal_all());
        //     trace!(?ty);
        //     self.layout_of(ty).llvm_type(self)
        // };
        // self.get_static_inner(def_id, llty)
        todo!()
    }
}
