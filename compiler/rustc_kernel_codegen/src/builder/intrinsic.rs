use rustc_codegen_ssa::{mir::{operand::OperandRef, place::PlaceRef}, traits::{ConstMethods, IntrinsicCallMethods}};
use rustc_middle::ty::{layout::HasTyCtxt, ParamEnv, Ty, TyKind};
use rustc_span::sym;
use rustc_target::abi::call::PassMode;
use rustc_codegen_ssa::traits::BuilderMethods;

use crate::function::FunctionNVVM;

use super::Builder;

impl<'tcx> IntrinsicCallMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn codegen_intrinsic_call(
        &mut self,
        instance: rustc_middle::ty::Instance<'tcx>,
        fn_abi: &rustc_target::abi::call::FnAbi<'tcx, Ty<'tcx>>,
        args: &[rustc_codegen_ssa::mir::operand::OperandRef<'tcx, Self::Value>],
        llresult: Self::Value,
        span: rustc_span::Span,
    ) -> Result<(), rustc_middle::ty::Instance<'tcx>> {
        //todo!()
        let tcx = self.tcx();
        let callee_ty = instance.ty(tcx, ParamEnv::reveal_all());

        let TyKind::FnDef(def_id, fn_args) = *callee_ty.kind() else {
            panic!("expected fn item type, found {}", callee_ty);
        };

        let sig = callee_ty.fn_sig(tcx);
        let sig = tcx.normalize_erasing_late_bound_regions(ParamEnv::reveal_all(), sig);
        let arg_tys = sig.inputs();
        let ret_ty = sig.output();
        let name = tcx.item_name(def_id);

        let llret_ty = self.lower_ty(&ret_ty);
        let result = PlaceRef::new_sized(llresult, fn_abi.ret.layout);

        //let simple = get_simple_intrinsic(self, name);
        
        let val = match name {
            sym::unlikely => self
            .call_intrinsic("llvm.expect.i1", &[args[0].immediate(), self.const_bool(false)]),
            sym::ctpop => {
                let arg = args[0].immediate();
                self.call_intrinsic("llvm.ctpop", &[arg])
            }
            sym::expf64 => {
                let arg = args[0].immediate();
                (self).call_intrinsic("__nv_exp", &[arg])
            }
            sym::sinf64 => {
                let arg = args[0].immediate();
                (self).call_intrinsic("__nv_sin", &[arg])
            }
            sym::cosf64 => {
                let arg = args[0].immediate();
                (self).call_intrinsic("__nv_cos", &[arg])
            }
            _ => panic!("unknown intrinsic '{}'", name),
        };

        if !fn_abi.ret.is_ignore() {
            if let PassMode::Cast { .. } = &fn_abi.ret.mode {
                self.store(val, result.val.llval, result.val.align);
            } else {
                OperandRef::from_immediate_or_packed_pair(self, val, result.layout)
                    .val
                    .store(self, result);
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

    fn type_test(&mut self, pointer: Self::Value, typeid: Self::Value) -> Self::Value {
        todo!()
    }

    fn type_checked_load(
        &mut self,
        llvtable: Self::Value,
        vtable_byte_offset: u64,
        typeid: Self::Value,
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
