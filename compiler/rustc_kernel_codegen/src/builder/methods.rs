use rustc_middle::{bug, ty::{layout::HasTyCtxt, Ty, TyCtxt}};
use rustc_codegen_ssa::{mir::{operand::{OperandRef, OperandValue}, place::PlaceRef}, traits::{BaseTypeMethods, BuilderMethods, ConstMethods, LayoutTypeMethods, OverflowOp}};
use rustc_span::symbol::kw::In;
use rustc_target::abi::{call::FnAbi, Abi, Align, Scalar, Size, WrappingRange};
use crate::{basic_block::BasicBlock, ty::TypeNVVM, value::{Comp, Instruction, Val, ValueNVVM}};
use crate::codegen_cx::abi::LayoutExt;

use super::Builder;

impl<'a, 'm, 'tcx> BuilderMethods<'a, 'tcx> for Builder<'a, 'm, 'tcx> {
    fn build(cx: &'a Self::CodegenCx, llbb: Self::BasicBlock) -> Self {
        Self {
            codegen_cx: cx,
            basic_block: llbb,
        }
    }

    fn cx(&self) -> &Self::CodegenCx {
        self.codegen_cx
    }

    fn llbb(&self) -> Self::BasicBlock {
        todo!()
    }

    fn set_span(&mut self, span: rustc_span::Span) {
        // TODO!
    }

    fn append_block(cx: &'a Self::CodegenCx, llfn: Self::Function, name: &str) -> Self::BasicBlock {
        //println!("append_block, name: {}", name);
        let bb = BasicBlock::new(llfn, name);
        let bbref = cx.get_module().arena.alloc(bb);
        llfn.add_basic_block(bbref);
        bbref
    }

    fn append_sibling_block(&mut self, name: &str) -> Self::BasicBlock {
        // TODO: probably wrong
        Self::append_block(self.codegen_cx, self.basic_block.func, name)
    }

    fn switch_to_block(&mut self, llbb: Self::BasicBlock) {
        self.basic_block = llbb;
    }

    fn ret_void(&mut self) {
        // build a return void instruction
        let r = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(Instruction::Retvoid), None);
        
        // add the instruction to the current basic block
        self.basic_block.add_instr(r);
    }

    fn ret(&mut self, v: Self::Value) {
        // build a return instruction
        let r = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(Instruction::Ret(v)), None);
        
        // add the instruction to the current basic block
        self.basic_block.add_instr(r);
    }

    fn br(&mut self, dest: Self::BasicBlock) {
        todo!()
    }

    fn cond_br(
        &mut self,
        cond: Self::Value,
        then_llbb: Self::BasicBlock,
        else_llbb: Self::BasicBlock,
    ) {
        let instr = Instruction::ConditionalBranch{ cond, true_block: then_llbb, false_block: else_llbb };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), None);
        
        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
    }

    fn switch(
        &mut self,
        v: Self::Value,
        else_llbb: Self::BasicBlock,
        cases: impl ExactSizeIterator<Item = (u128, Self::BasicBlock)>,
    ) {
        todo!()
    }

    fn invoke(
        &mut self,
        llty: Self::Type,
        fn_attrs: Option<&rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs>,
        fn_abi: Option<&FnAbi<'tcx, Ty<'tcx>>>,
        llfn: Self::Value,
        args: &[Self::Value],
        then: Self::BasicBlock,
        catch: Self::BasicBlock,
        funclet: Option<&Self::Funclet>,
        instance: Option<rustc_middle::ty::Instance<'tcx>>,
    ) -> Self::Value {
        todo!()
    }

    fn unreachable(&mut self) {
        let instr = Instruction::Unreachable;
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), None);
        self.basic_block.add_instr(v);
    }

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = Instruction::Add(lhs, rhs);
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(self.cx().val_ty(lhs)));
        
        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn fadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fadd_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fadd_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn sub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = Instruction::Sub(lhs, rhs);
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(self.cx().val_ty(lhs)));
        
        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn fsub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fsub_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fsub_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn mul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fmul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fmul_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fmul_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn udiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn exactudiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn sdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn exactsdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fdiv_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn fdiv_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn urem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn srem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn frem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn frem_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn frem_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn shl(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn lshr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn ashr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn unchecked_sadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn unchecked_uadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn unchecked_ssub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn unchecked_usub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn unchecked_smul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn unchecked_umul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn and(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = Instruction::And(lhs, rhs);
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(self.cx().val_ty(lhs)));
        
        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn or(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = Instruction::Or(lhs, rhs);
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(self.cx().val_ty(lhs)));

        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn xor(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = Instruction::Xor(lhs, rhs);
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(self.cx().val_ty(lhs)));
        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn neg(&mut self, v: Self::Value) -> Self::Value {
        todo!()
    }

    fn fneg(&mut self, v: Self::Value) -> Self::Value {
        todo!()
    }

    fn not(&mut self, v: Self::Value) -> Self::Value {
        todo!()
    }

    fn checked_binop(
        &mut self,
        oop: OverflowOp,
        ty: Ty<'_>,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> (Self::Value, Self::Value) {
        //TODO make this a checked operation
        // println!(
        //     "Checked binop `{:?}`, lhs: `{:?}`, rhs: `{:?}`",
        //     ty,
        //     lhs,
        //     rhs
        // );
        use rustc_middle::ty::IntTy::*;
        use rustc_middle::ty::UintTy::*;
        use rustc_middle::ty::{Int, Uint};

        let new_kind = match ty.kind() {
            Int(t @ Isize) => Int(t.normalize(self.tcx().sess.target.pointer_width)),
            Uint(t @ Usize) => Uint(t.normalize(self.tcx().sess.target.pointer_width)),
            t @ (Uint(_) | Int(_)) => t.clone(),
            _ => panic!("tried to get overflow intrinsic for op applied to non-int type"),
        };

        let name = match oop {
            OverflowOp::Add => match new_kind {
                Int(I8) => "__nvvm_i8_addo",
                Int(I16) => "llvm.sadd.with.overflow.i16",
                Int(I32) => "llvm.sadd.with.overflow.i32",
                Int(I64) => "llvm.sadd.with.overflow.i64",
                Int(I128) => "__nvvm_i128_addo",

                Uint(U8) => "__nvvm_u8_addo",
                Uint(U16) => "llvm.uadd.with.overflow.i16",
                Uint(U32) => "llvm.uadd.with.overflow.i32",
                Uint(U64) => "llvm.uadd.with.overflow.i64",
                Uint(U128) => "__nvvm_u128_addo",
                _ => unreachable!(),
            },
            OverflowOp::Sub => match new_kind {
                Int(I8) => "__nvvm_i8_subo",
                Int(I16) => "llvm.ssub.with.overflow.i16",
                Int(I32) => "llvm.ssub.with.overflow.i32",
                Int(I64) => "llvm.ssub.with.overflow.i64",
                Int(I128) => "__nvvm_i128_subo",

                Uint(U8) => "__nvvm_u8_subo",
                Uint(U16) => "llvm.usub.with.overflow.i16",
                Uint(U32) => "llvm.usub.with.overflow.i32",
                Uint(U64) => "llvm.usub.with.overflow.i64",
                Uint(U128) => "__nvvm_u128_subo",

                _ => unreachable!(),
            },
            OverflowOp::Mul => match new_kind {
                Int(I8) => "__nvvm_i8_mulo",
                Int(I16) => "llvm.smul.with.overflow.i16",
                Int(I32) => "llvm.smul.with.overflow.i32",
                Int(I64) => "llvm.smul.with.overflow.i64",
                Int(I128) => "__nvvm_i128_mulo",

                Uint(U8) => "__nvvm_u8_mulo",
                Uint(U16) => "llvm.umul.with.overflow.i16",
                Uint(U32) => "llvm.umul.with.overflow.i32",
                Uint(U64) => "llvm.umul.with.overflow.i64",
                Uint(U128) => "__nvvm_u128_mulo",

                _ => unreachable!(),
            },
        };

        let res = self.call_intrinsic(name, &[lhs, rhs]);
        (self.extract_value(res, 0), self.extract_value(res, 1))
    }

    fn from_immediate(&mut self, val: Self::Value) -> Self::Value {
        if self.cx().val_ty(val) == self.cx().type_i1() {
            self.zext(val, self.cx().type_i8())
        } else {
            val
        }
    }

    fn to_immediate_scalar(&mut self, val: Self::Value, scalar: Scalar) -> Self::Value {
        if scalar.is_bool() {
            return self.trunc(val, self.cx().type_i1());
        }
        val
    }

    fn alloca(&mut self, size: Size, align: Align) -> Self::Value {
        //println!("Alloca size: {:?}, align: {:?}", size, align);
        // create a new alloca instruction
        let ty = self.cx().type_i8();
        let ptr_ty = self.cx().type_pointer(ty);
        let alloca = ValueNVVM::Instr(Instruction::Alloca(ty, size.bytes()));
        let v = self.cx().get_module_mut().
            create_val(alloca, Some(ptr_ty)); // the type of the alloca is not known

        // add the alloca instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn dynamic_alloca(&mut self, size: Self::Value, align: Align) -> Self::Value {
        todo!()
    }

    fn load(&mut self, ty: Self::Type, ptr: Self::Value, align: Align) -> Self::Value {
        let instr = Instruction::Load { ty, ptr, align: align.bytes() };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(ty));
        self.basic_block.add_instr(v);
        v
    }

    fn volatile_load(&mut self, ty: Self::Type, ptr: Self::Value) -> Self::Value {
        todo!()
    }

    fn atomic_load(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        size: Size,
    ) -> Self::Value {
        todo!()
    }

    fn load_operand(&mut self, place: PlaceRef<'tcx, Self::Value>) -> OperandRef<'tcx, Self::Value> {
        if place.layout.is_zst() {
            return OperandRef::zero_sized(place.layout);
        }

        let val = if let Some(_) = place.val.llextra {
            // FIXME: Merge with the `else` below?
            OperandValue::Ref(place.val)
        } else if place.layout.is_immediate() {
            let mut const_llval = None;
            let llty = self.cx().backend_type(place.layout);
            /*unsafe {
                if let Some(global) = llvm::LLVMIsAGlobalVariable(place.val.llval) {
                    if llvm::LLVMIsGlobalConstant(global) == llvm::True {
                        if let Some(init) = llvm::LLVMGetInitializer(global) {
                            if self.val_ty(init) == llty {
                                const_llval = Some(init);
                            }
                        }
                    }
                }
            }*/
            let llval = const_llval.unwrap_or_else(|| {
                let load = self.load(llty, place.val.llval, place.val.align);
                // if let Abi::Scalar(scalar) = place.layout.abi {
                //     scalar_load_metadata(self, load, scalar, place.layout, Size::ZERO);
                // }
                load
            });
            OperandValue::Immediate(self.to_immediate(llval, place.layout))
        } else if let Abi::ScalarPair(a, b) = place.layout.abi {
            let b_offset = a.size(self).align_to(b.align(self).abi);

            let mut load = |i, scalar: rustc_target::abi::Scalar, layout, align, offset| {
                let llptr = if i == 0 {
                    place.val.llval
                } else {
                    self.inbounds_ptradd(place.val.llval, self.cx().const_usize(b_offset.bytes()))
                };
                
                // get the type of the first scalar
                let ty = self.cx().scalar_pair_element_backend_type(layout, i, false);

                let load = self.load(ty, llptr, align);
                //scalar_load_metadata(self, load, scalar, layout, offset);
                self.to_immediate_scalar(load, scalar)
            };

            OperandValue::Pair(
                load(0, a, place.layout, place.val.align, Size::ZERO),
                load(1, b, place.layout, place.val.align.restrict_for_offset(b_offset), b_offset),
            )
        } else {
            OperandValue::Ref(place.val)
        };

        OperandRef { val, layout: place.layout }
    }

    fn write_operand_repeatedly(
        &mut self,
        elem: rustc_codegen_ssa::mir::operand::OperandRef<'tcx, Self::Value>,
        count: u64,
        dest: rustc_codegen_ssa::mir::place::PlaceRef<'tcx, Self::Value>,
    ) {
        todo!()
    }

    fn range_metadata(&mut self, load: Self::Value, range: WrappingRange) {
        todo!()
    }

    fn nonnull_metadata(&mut self, load: Self::Value) {
        todo!()
    }

    fn store(&mut self, val: Self::Value, ptr: Self::Value, align: Align) -> Self::Value {
        self.store_with_flags(val, ptr, align, rustc_codegen_ssa::MemFlags::empty())
    }

    fn store_with_flags(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        align: Align,
        flags: rustc_codegen_ssa::MemFlags,
    ) -> Self::Value {
        // check if the type of the pointer is the same as the type of the value
        // if not bitcast the value to the type of the pointer
        let target_ty = self.cx().type_pointer(self.cx().val_ty(val));
        let org_ptr_ty = self.cx().val_ty(ptr);

        // if the types are not the same, cast the value to the type of the pointer
        let ptr = if target_ty != org_ptr_ty {
            let cast = Instruction::BitCast { ty: org_ptr_ty, val: ptr, to: target_ty };
            let castval = self.cx().get_module_mut().create_val(ValueNVVM::Instr(cast), Some(target_ty));
            self.basic_block.add_instr(castval);
            castval
        } else { ptr };

        // build a store instruction
        let instr = Instruction::Store { val, ptr, align: align.bytes() };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), None);

        
        

        // add the store instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn atomic_store(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        size: Size,
    ) {
        todo!()
    }

    fn gep(&mut self, ty: Self::Type, ptr: Self::Value, indices: &[Self::Value]) -> Self::Value {
        todo!()
    }

    fn inbounds_gep(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        indices: &[Self::Value],
    ) -> Self::Value {
        //println!("Inbounds GEP, ty: {:?}, ptr: {:?}, indices: {:?}", ty, ptr, indices);

        // infer the return type 
        // we don't need to check the first index, because it is always a pointer
        let mut rty = ty.clone(); 
        for i in indices[1..indices.len()].iter() {
            match rty.0 {
                TypeNVVM::Pointer(t) => rty = t.clone(),
                TypeNVVM::Array(t, _) => rty = t.clone(),
                TypeNVVM::Struct(types) => {
                    let ValueNVVM::Constant(crate::value::Const::I(idx)) = i.0 else {
                        bug!("Invalid index for GEP");
                    };
                    rty = types[*idx as usize].clone();
                }
                _ => panic!("Invalid type for GEP"),
            }
        }

        // add a pointer type to the return type
        rty = self.cx().type_pointer(rty);

        // build an inbounds getelementptr instruction
        let instr = Instruction::InBoundsGep { ty, ptr, indices: indices.to_vec() };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(rty));

        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn trunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        // truncate the value to the destination type
        let instr = Instruction::Trunc { val, to: dest_ty };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(dest_ty));
        self.basic_block.add_instr(v);
        v
    }

    fn sext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        // sign extend the value to the destination type
        let instr = Instruction::SExt { val, to: dest_ty };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(dest_ty));
        self.basic_block.add_instr(v);
        v
    }

    fn fptoui_sat(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn fptosi_sat(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn fptoui(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn fptosi(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn uitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn sitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn fptrunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn fpext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn ptrtoint(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.bitcast(val, dest_ty)
    }

    fn inttoptr(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.bitcast(val, dest_ty)
    }

    fn bitcast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        // build a bitcast instruction
        let ty = self.cx().val_ty(val);
        let instr = Instruction::BitCast { ty, val, to: dest_ty };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(dest_ty));
        self.basic_block.add_instr(v);
        v
    }

    fn intcast(&mut self, val: Self::Value, dest_ty: Self::Type, is_signed: bool) -> Self::Value {
        // get the original type
        let ty = self.cx().val_ty(val); 
        // if the original type is larger than the destination type, truncate
        // if the original type is smaller than the destination type, sign or zero extend
        let tysz1 = ty.size();
        let tysz2 = dest_ty.size();
        if tysz1 > tysz2 {
            self.trunc(val, dest_ty)
        } else if tysz1 < tysz2 {
            if is_signed {
                self.sext(val, dest_ty)
            } else {
                self.zext(val, dest_ty)
            }
        } else {
            self.bitcast(val, dest_ty)
        }
    }

    fn pointercast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.bitcast(val, dest_ty)
    }

    fn icmp(&mut self, op: rustc_codegen_ssa::common::IntPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = Instruction::ICmp(Comp::from(op), lhs, rhs);
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(self.cx().type_i1()));
        self.basic_block.add_instr(v);
        v
    }

    fn fcmp(&mut self, op: rustc_codegen_ssa::common::RealPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!()
    }

    fn memcpy(
        &mut self,
        dst: Self::Value,
        dst_align: Align,
        src: Self::Value,
        src_align: Align,
        size: Self::Value,
        flags: rustc_codegen_ssa::MemFlags,
    ) {
        todo!()
    }

    fn memmove(
        &mut self,
        dst: Self::Value,
        dst_align: Align,
        src: Self::Value,
        src_align: Align,
        size: Self::Value,
        flags: rustc_codegen_ssa::MemFlags,
    ) {
        todo!()
    }

    fn memset(
        &mut self,
        ptr: Self::Value,
        fill_byte: Self::Value,
        size: Self::Value,
        align: Align,
        flags: rustc_codegen_ssa::MemFlags,
    ) {
        todo!()
    }

    fn select(
        &mut self,
        cond: Self::Value,
        then_val: Self::Value,
        else_val: Self::Value,
    ) -> Self::Value {
        todo!()
    }

    fn va_arg(&mut self, list: Self::Value, ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn extract_element(&mut self, vec: Self::Value, idx: Self::Value) -> Self::Value {
        todo!()
    }

    fn vector_splat(&mut self, num_elts: usize, elt: Self::Value) -> Self::Value {
        todo!()
    }

    fn extract_value(&mut self, agg_val: Self::Value, idx: u64) -> Self::Value {
        // create a new extract value instruction
        // get the type of the aggregate value
        let ty = self.cx().val_ty(agg_val);

        // derive the result type of the extract value instruction
        let subty = match ty.0 {
            TypeNVVM::Struct(els) => els[idx as usize],
            _ => panic!("Expected struct type, found {:?}", ty),
        };

        let instr = Instruction::ExtractValue(ty, agg_val, idx);
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(subty));

        // add the instruction to the current basic block
        self.basic_block.add_instr(v);
        v
    }

    fn insert_value(&mut self, agg_val: Self::Value, elt: Self::Value, idx: u64) -> Self::Value {
        todo!()
    }

    fn set_personality_fn(&mut self, personality: Self::Value) {
        todo!()
    }

    fn cleanup_landing_pad(&mut self, pers_fn: Self::Value) -> (Self::Value, Self::Value) {
        todo!()
    }

    fn filter_landing_pad(&mut self, pers_fn: Self::Value) -> (Self::Value, Self::Value) {
        todo!()
    }

    fn resume(&mut self, exn0: Self::Value, exn1: Self::Value) {
        todo!()
    }

    fn cleanup_pad(&mut self, parent: Option<Self::Value>, args: &[Self::Value]) -> Self::Funclet {
        todo!()
    }

    fn cleanup_ret(&mut self, funclet: &Self::Funclet, unwind: Option<Self::BasicBlock>) {
        todo!()
    }

    fn catch_pad(&mut self, parent: Self::Value, args: &[Self::Value]) -> Self::Funclet {
        todo!()
    }

    fn catch_switch(
        &mut self,
        parent: Option<Self::Value>,
        unwind: Option<Self::BasicBlock>,
        handlers: &[Self::BasicBlock],
    ) -> Self::Value {
        todo!()
    }

    fn atomic_cmpxchg(
        &mut self,
        dst: Self::Value,
        cmp: Self::Value,
        src: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        failure_order: rustc_codegen_ssa::common::AtomicOrdering,
        weak: bool,
    ) -> (Self::Value, Self::Value) {
        todo!()
    }

    fn atomic_rmw(
        &mut self,
        op: rustc_codegen_ssa::common::AtomicRmwBinOp,
        dst: Self::Value,
        src: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
    ) -> Self::Value {
        todo!()
    }

    fn atomic_fence(&mut self, order: rustc_codegen_ssa::common::AtomicOrdering, scope: rustc_codegen_ssa::common::SynchronizationScope) {
        todo!()
    }

    fn set_invariant_load(&mut self, load: Self::Value) {
        todo!()
    }

    fn lifetime_start(&mut self, ptr: Self::Value, size: Size) {
        todo!()
    }

    fn lifetime_end(&mut self, ptr: Self::Value, size: Size) {
        todo!()
    }

    fn instrprof_increment(
        &mut self,
        fn_name: Self::Value,
        hash: Self::Value,
        num_counters: Self::Value,
        index: Self::Value,
    ) {
        todo!()
    }

    fn call(
        &mut self,
        llty: Self::Type,
        fn_attrs: Option<&rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs>,
        fn_abi: Option<&FnAbi<'tcx, Ty<'tcx>>>,
        llfn: Self::Value,
        args: &[Self::Value],
        funclet: Option<&Self::Funclet>,
        instance: Option<rustc_middle::ty::Instance<'tcx>>,
    ) -> Self::Value {

        let TypeNVVM::Fn(_, ret) = llty.0 else {
            bug!("Expected function type, found {:?}", llty);
        };

        let mut args_vec = Vec::new();
        // check if all function parameters have the correct type
        for (i, (arg, expected_ty)) in args.iter().zip(fn_abi.unwrap().args.iter()).enumerate() {
            let ty = self.cx().backend_type(expected_ty.layout);
            if self.cx().val_ty(*arg) != ty {
                // perfrom a bitcast if the types do not match
                let cast = Instruction::BitCast { ty: self.cx().val_ty(*arg), val: *arg, to: ty };
                let v = self.cx().get_module_mut().
                    create_val(ValueNVVM::Instr(cast), Some(ty));
                self.basic_block.add_instr(v);
                args_vec.push(v);
            } else {
                args_vec.push(*arg);
            }
        }


        let instr = Instruction::Call { 
            ret_ty: *ret,
            fn_val: llfn, 
            fn_ty: llty,
            args: args_vec,
        };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(*ret));
        self.basic_block.add_instr(v);
        v
    }

    fn zext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        // build a zero extension instruction
        let instr = Instruction::ZExt { val, to: dest_ty };
        let v = self.cx().get_module_mut().
            create_val(ValueNVVM::Instr(instr), Some(dest_ty));
        self.basic_block.add_instr(v);
        v
    }

    fn apply_attrs_to_cleanup_callsite(&mut self, llret: Self::Value) {
        todo!()
    }
}

impl<'a, 'm, 'tcx> Builder<'a, 'm, 'tcx> {
    fn call_intrinsic(&self, name: &str, args: &[Val<'m>]) -> Val<'m> {
        let module = self.cx().get_module_mut();
        module.use_intrinsic(name);
        let fn_val = if let Some(intr) = module.get_intrinsic(name) {
            intr
        } else {
            bug!("Unknown intrinsic '{}'", name)
        };
        let fn_ty = *module.valtypes.get(&fn_val).unwrap();
        let TypeNVVM::Fn(args, ret) = fn_ty.0 else {
            bug!("Expected function type, found {:?}", fn_ty);
        };
        
        let args = args.iter().enumerate().map(|(i, ty)| {
            module.create_val(ValueNVVM::Param { 
                    func_name: name.to_string(), 
                    idx: i, 
                    ty: *ty 
                }, 
            Some(*ty))
        }).collect::<Vec<_>>();

        let instr = Instruction::Call { 
            ret_ty: *ret,
            fn_val,
            fn_ty, 
            args,
        };
        let v = module.create_val(ValueNVVM::Instr(instr), Some(*ret));
        self.basic_block.add_instr(v);
        v
    }
}

