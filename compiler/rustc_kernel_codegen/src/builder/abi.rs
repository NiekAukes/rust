use rustc_codegen_ssa::{mir::{operand::{OperandRef, OperandValue}, place::{PlaceRef, PlaceValue}}, traits::{AbiBuilderMethods, ArgAbiMethods, BuilderMethods}, MemFlags};
use rustc_middle::{bug, ty::{layout::{FnAbiOfHelpers, LayoutOfHelpers}, Ty}};
use rustc_target::abi::call::{ArgAbi, PassMode};
use rustc_codegen_ssa::traits::ConstMethods;

use crate::{codegen_cx::CodegenCx, ty::TyNVVM, value::Val};

use super::Builder;

impl<'tcx> AbiBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn get_param(&mut self, index: usize) -> Self::Value {
        // get the parameter of the function at the given index
        match self.basic_block.func.args.get(index) {
            Some(param) => *param,
            None => panic!("Parameter not found at index {}", index),
        }
    }
}

impl<'m, 'tcx> ArgAbiMethods<'tcx> for Builder<'_, 'm, 'tcx> {
    fn store_fn_arg(
        &mut self,
        arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        idx: &mut usize,
        dst: PlaceRef<'tcx, Self::Value>,
    ) {
        arg_abi.store_fn_arg(self, idx, dst)
    }
    fn store_arg(
        &mut self,
        arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        val: Val<'m>,
        dst: PlaceRef<'tcx, Val<'m>>,
    ) {
        arg_abi.store(self, val, dst)
    }
    fn arg_memory_ty(&self, arg_abi: &ArgAbi<'tcx, Ty<'tcx>>) -> TyNVVM<'m> {
        arg_abi.memory_ty(self)
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
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

impl<'tcx> LayoutOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
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


pub trait ArgAbiExt<'m, 'tcx> {
    fn memory_ty(&self, cx: &CodegenCx<'m, 'tcx>) -> TyNVVM<'m>;
    fn store(
        &self,
        bx: &mut Builder<'_, 'm, 'tcx>,
        val: Val<'m>,
        dst: PlaceRef<'tcx, Val<'m>>,
    );
    fn store_fn_arg(
        &self,
        bx: &mut Builder<'_, 'm, 'tcx>,
        idx: &mut usize,
        dst: PlaceRef<'tcx, Val<'m>>,
    );
}

impl<'m, 'tcx> ArgAbiExt<'m, 'tcx> for ArgAbi<'tcx, Ty<'tcx>> {
    /// Gets the LLVM type for a place of the original Rust type of
    /// this argument/return, i.e., the result of `type_of::type_of`.
    fn memory_ty(&self, cx: &CodegenCx<'m, 'tcx>) -> TyNVVM<'m> {
        cx.lower_ty(&self.layout.ty)
    }

    /// Stores a direct/indirect value described by this ArgAbi into a
    /// place for the original Rust type of this argument/return.
    /// Can be used for both storing formal arguments into Rust variables
    /// or results of call/invoke instructions into their destinations.
    fn store(
        &self,
        bx: &mut Builder<'_, 'm, 'tcx>,
        val: Val<'m>,
        dst: PlaceRef<'tcx, Val<'m>>,
    ) {
        match &self.mode {
            PassMode::Ignore => {}
            // Sized indirect arguments
            PassMode::Indirect { attrs, meta_attrs: None, on_stack: _ } => {
                let align = attrs.pointee_align.unwrap_or(self.layout.align.abi);
                OperandValue::Ref(PlaceValue::new_sized(val, align)).store(bx, dst);
            }
            // Unsized indirect qrguments
            PassMode::Indirect { attrs: _, meta_attrs: Some(_), on_stack: _ } => {
                bug!("unsized `ArgAbi` must be handled through `store_fn_arg`");
            }
            PassMode::Cast { cast, pad_i32: _ } => {
                // The ABI mandates that the value is passed as a different struct representation.
                // Spill and reload it from the stack to convert from the ABI representation to
                // the Rust representation.
                let scratch_size = cast.size(bx);
                let scratch_align = cast.align(bx);
                // Note that the ABI type may be either larger or smaller than the Rust type,
                // due to the presence or absence of trailing padding. For example:
                // - On some ABIs, the Rust layout { f64, f32, <f32 padding> } may omit padding
                //   when passed by value, making it smaller.
                // - On some ABIs, the Rust layout { u16, u16, u16 } may be padded up to 8 bytes
                //   when passed by value, making it larger.
                let copy_bytes = std::cmp::min(scratch_size.bytes(), self.layout.size.bytes());
                // Allocate some scratch space...
                let llscratch = bx.alloca(scratch_size, scratch_align);
                bx.lifetime_start(llscratch, scratch_size);
                // ...store the value...
                bx.store(val, llscratch, scratch_align);
                // ... and then memcpy it to the intended destination.
                bx.memcpy(
                    dst.val.llval,
                    self.layout.align.abi,
                    llscratch,
                    scratch_align,
                    bx.const_usize(copy_bytes),
                    MemFlags::empty(),
                );
                bx.lifetime_end(llscratch, scratch_size);
            }
            _ => {
                OperandRef::from_immediate_or_packed_pair(bx, val, self.layout).val.store(bx, dst);
            }
        }
    }

    fn store_fn_arg(
        &self,
        bx: &mut Builder<'_, 'm, 'tcx>,
        idx: &mut usize,
        dst: PlaceRef<'tcx, Val<'m>>,
    ) {
        let mut next = || {
            let val = bx.get_param(*idx);
            *idx += 1;
            val
        };
        match self.mode {
            PassMode::Ignore => {}
            PassMode::Pair(..) => {
                OperandValue::Pair(next(), next()).store(bx, dst);
            }
            PassMode::Indirect { attrs: _, meta_attrs: Some(_), on_stack: _ } => {
                let place_val = PlaceValue {
                    llval: next(),
                    llextra: Some(next()),
                    align: self.layout.align.abi,
                };
                OperandValue::Ref(place_val).store(bx, dst);
            }
            PassMode::Direct(_)
            | PassMode::Indirect { attrs: _, meta_attrs: None, on_stack: _ }
            | PassMode::Cast { .. } => {
                let next_arg = next();
                self.store(bx, next_arg, dst);
            }
        }
    }
}