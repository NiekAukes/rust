use rustc_codegen_ssa::mir::operand::OperandRef;
use rustc_codegen_ssa::mir::place::PlaceRef;
use rustc_codegen_ssa::traits::{BuilderMethods, ConstCodegenMethods, IntrinsicCallBuilderMethods};
use rustc_middle::ty::layout::{FnAbiOf, HasTyCtxt};
use rustc_middle::ty::{Instance, List, ParamEnv, Ty, TyKind, TypingEnv};
use rustc_span::{Span, sym};
use rustc_target::callconv::{FnAbi, PassMode};

use super::Builder;
use crate::function::FunctionNVVM;

impl<'tcx> IntrinsicCallBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn codegen_intrinsic_call(
        &mut self,
        instance: Instance<'tcx>,
        args: &[OperandRef<'tcx, Self::Value>],
        llresult: PlaceRef<'tcx, Self::Value>,
        span: Span,
    ) -> Result<(), rustc_middle::ty::Instance<'tcx>> {
        //todo!()
        let tcx = self.tcx();
        let callee_ty = instance.ty(tcx, TypingEnv::fully_monomorphized());

        let TyKind::FnDef(def_id, fn_args) = *callee_ty.kind() else {
            panic!("expected fn item type, found {}", callee_ty);
        };

        let sig = callee_ty.fn_sig(tcx);
        let sig = tcx.normalize_erasing_late_bound_regions(TypingEnv::fully_monomorphized(), sig);
        let arg_tys = sig.inputs();
        let ret_ty = sig.output();
        let name = tcx.item_name(def_id);

        let llret_ty = self.lower_ty(&ret_ty);

        let fn_abi = self.cx().fn_abi_of_instance(instance, List::empty());

        //let result = PlaceRef::new_sized(llresult, fn_abi.ret.layout);

        //let simple = get_simple_intrinsic(self, name);

        let val = match name {
            sym::unlikely => self
                .call_intrinsic("llvm.expect.i1", &[args[0].immediate(), self.const_bool(false)]),
            sym::ctpop => {
                let arg = args[0].immediate();
                self.call_intrinsic("llvm.ctpop", &[arg])
            }
            _ => panic!("unknown intrinsic '{}'", name),
        };

        if !fn_abi.ret.is_ignore() {
            if let PassMode::Cast { .. } = &fn_abi.ret.mode {
                self.store(val, llresult.val.llval, llresult.val.align);
            } else {
                OperandRef::from_immediate_or_packed_pair(self, val, llresult.layout)
                    .val
                    .store(self, llresult);
            }
        }
        Ok(())
    }

    fn abort(&mut self) {
        self.call_intrinsic("llvm.trap", &[]);
    }

    fn assume(&mut self, val: Self::Value) {
        self.call_intrinsic("llvm.assume", &[val]);
    }

    fn expect(&mut self, cond: Self::Value, expected: bool) -> Self::Value {
        todo!()
    }

    fn type_checked_load(
        &mut self,
        llvtable: Self::Value,
        vtable_byte_offset: u64,
        metadata: (),
    ) -> Self::Value {
        todo!()
    }

    fn va_start(&mut self, val: Self::Value) -> Self::Value {
        todo!()
    }

    fn va_end(&mut self, val: Self::Value) -> Self::Value {
        todo!()
    }
}
