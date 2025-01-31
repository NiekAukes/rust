use std::borrow::Cow;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::collections::HashMap;

use rustc_codegen_ssa::traits::AsmMethods;
use rustc_codegen_ssa::traits::BackendTypes;
use rustc_codegen_ssa::traits::BaseTypeMethods;
use rustc_codegen_ssa::traits::CodegenMethods;
use rustc_codegen_ssa::traits::DebugInfoMethods;
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::intern::Interned;
use rustc_middle::ty::layout::HasParamEnv;
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::Ty;
use rustc_middle::ty::TyCtxt;
use rustc_target::abi::VariantIdx;
use rustc_target::spec::HasTargetSpec;
use rustc_target::spec::Target;
use crate::function::FunctionNVVM;
use crate::module::ModuleNVVM;
use crate::ty::{TyNVVM, TypeNVVM};
use crate::basic_block::BasicBlock;
use crate::value::Val;
use crate::value::ValueNVVM;

mod declare;
mod consts;
mod statics;
mod debug;
mod misc;
pub mod abi;

pub struct CodegenCx<'m, 'tcx> {
    // ...
    tcx: TyCtxt<'tcx>,
    // unsafe cell to allow mutation of the module
    module: UnsafeCell<ModuleNVVM<'m>>,

    pub(crate) typecache: UnsafeCell<FxHashMap<Ty<'tcx>, TyNVVM<'m>>>,

    //session: &'tcx rustc_session::Session,
    pub(crate) eh_personality: Cell<Option<Val<'m>>>,

    target: &'tcx Target,
}

impl<'m> BackendTypes for CodegenCx<'m, '_> {
    type Value = Val<'m>;
    // FIXME(eddyb) replace this with a `Function` "subclass" of `Value`.
    type Function = &'m FunctionNVVM<'m>;

    type BasicBlock = &'m BasicBlock<'m>;
    type Type = TyNVVM<'m>;
    type Funclet = ();

    type DIScope = ();
    type DILocation = ();
    type DIVariable = ();
}

//impl<'tcx> CodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {}

impl HasTargetSpec for CodegenCx<'_, '_> {
    fn target_spec(&self) -> &rustc_target::spec::Target {
        self.target
    }
}

impl<'tcx> AsmMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn codegen_global_asm(
        &self,
        template: &[rustc_ast::InlineAsmTemplatePiece],
        operands: &[rustc_codegen_ssa::traits::GlobalAsmOperandRef<'tcx>],
        options: rustc_ast::InlineAsmOptions,
        line_spans: &[rustc_span::Span],
    ) {
        todo!()
    }
}


impl<'tcx> HasParamEnv<'tcx> for CodegenCx<'_, 'tcx> {
    fn param_env(&self) -> rustc_middle::ty::ParamEnv<'tcx> {
        rustc_middle::ty::ParamEnv::reveal_all()
    }
}

impl<'tcx> HasTyCtxt<'tcx> for CodegenCx<'_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}


impl<'m, 'tcx> CodegenCx<'m, 'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, module: ModuleNVVM<'m>) -> Self {
        let mut target = tcx.sess.target.clone();
        target.arch = Cow::from("nvvm".to_string());
        let t = tcx.arena.dropless.alloc(target);
        Self {
            // ...
            tcx,
            module: UnsafeCell::new(module),
            typecache: UnsafeCell::new(FxHashMap::default()),
            eh_personality: Cell::new(None),
            target: t,
        }
    }

    pub fn finalize(self) -> ModuleNVVM<'m> {
        self.module.into_inner()
    }

    pub fn get_module(&self) -> &ModuleNVVM<'m> {
        unsafe { &*self.module.get() }
    }

    pub fn get_module_mut(&self) -> &mut ModuleNVVM<'m> {
        unsafe { &mut *self.module.get() }
    }

    pub fn build_intrinsics(&mut self) {
        let module = self.get_module_mut();
        macro_rules! ifn {
            ($($name:literal)|*, fn($($arg:expr),*) -> $ret:expr) => {
                for name in &[$($name),*] {
                    let value = ValueNVVM::FnRef(name.to_string());
                    let args = &[$($arg),*];
                    let ty = self.type_func(args, $ret);
                    let val = module.create_val(value, Some(ty));
                    module.intrinsics.insert(name.to_string(), val);
                }
            };
        }

        let i8p = self.type_ptr();
        let void = self.type_void();
        let i1 = self.type_i1();
        let t_i8 = self.type_i8();
        let t_i16 = self.type_i16();
        let t_i32 = self.type_i32();
        let t_i64 = self.type_i64();
        let t_f32 = self.type_f32();
        let t_f64 = self.type_f64();
        let t_isize = self.type_isize();

        let t_i8_i1 = self.type_struct(&[t_i8, i1], false);
        let t_i16_i1 = self.type_struct(&[t_i16, i1], false);
        let t_i32_i1 = self.type_struct(&[t_i32, i1], false);
        let t_i64_i1 = self.type_struct(&[t_i64, i1], false);

        let voidp = self.type_voidptr();
        
        ifn!("llvm.trap" | "llvm.sideeffect", fn() -> void);
        ifn!("llvm.assume", fn(i1) -> void);
        ifn!("llvm.prefetch", fn(i8p, t_i32, t_i32, t_i32) -> void);

        ifn!("llvm.sadd.with.overflow.i16", fn(t_i16, t_i16) -> t_i16_i1);
        ifn!("llvm.sadd.with.overflow.i32", fn(t_i32, t_i32) -> t_i32_i1);
        ifn!("llvm.sadd.with.overflow.i64", fn(t_i64, t_i64) -> t_i64_i1);

        ifn!("llvm.uadd.with.overflow.i16", fn(t_i16, t_i16) -> t_i16_i1);
        ifn!("llvm.uadd.with.overflow.i32", fn(t_i32, t_i32) -> t_i32_i1);
        ifn!("llvm.uadd.with.overflow.i64", fn(t_i64, t_i64) -> t_i64_i1);

        ifn!("llvm.ssub.with.overflow.i16", fn(t_i16, t_i16) -> t_i16_i1);
        ifn!("llvm.ssub.with.overflow.i32", fn(t_i32, t_i32) -> t_i32_i1);
        ifn!("llvm.ssub.with.overflow.i64", fn(t_i64, t_i64) -> t_i64_i1);

        ifn!("llvm.usub.with.overflow.i16", fn(t_i16, t_i16) -> t_i16_i1);
        ifn!("llvm.usub.with.overflow.i32", fn(t_i32, t_i32) -> t_i32_i1);
        ifn!("llvm.usub.with.overflow.i64", fn(t_i64, t_i64) -> t_i64_i1);

        ifn!("llvm.smul.with.overflow.i16", fn(t_i16, t_i16) -> t_i16_i1);
        ifn!("llvm.smul.with.overflow.i32", fn(t_i32, t_i32) -> t_i32_i1);
        ifn!("llvm.smul.with.overflow.i64", fn(t_i64, t_i64) -> t_i64_i1);

        ifn!("llvm.umul.with.overflow.i16", fn(t_i16, t_i16) -> t_i16_i1);
        ifn!("llvm.umul.with.overflow.i32", fn(t_i32, t_i32) -> t_i32_i1);
        ifn!("llvm.umul.with.overflow.i64", fn(t_i64, t_i64) -> t_i64_i1);

        /*let i128_checked_binops = [
            "__nvvm_i128_addo",
            "__nvvm_u128_addo",
            "__nvvm_i128_subo",
            "__nvvm_u128_subo",
            "__nvvm_i128_mulo",
            "__nvvm_u128_mulo"
        ];

        for binop in i128_checked_binops {
            /*map.insert(binop, (vec![t_i128, t_i128], t_i128_i1));
            let llfn_ty = self.type_func(&[t_i128, t_i128], t_i128_i1);
            remapped.insert(llfn_ty, (Some(real_t_i128_i1), vec![(0, real_t_i128), (1, real_t_i128)]));*/


        }*/

        /* 
        let i128_saturating_ops = [
            "llvm.sadd.sat.i128",
            "llvm.uadd.sat.i128",
            "llvm.ssub.sat.i128",
            "llvm.usub.sat.i128",
        ];

        for binop in i128_saturating_ops {
            map.insert(binop, (vec![t_i128, t_i128], t_i128));
            let llfn_ty = self.type_func(&[t_i128, t_i128], t_i128);
            remapped.insert(llfn_ty, (Some(real_t_i128), vec![(0, real_t_i128), (1, real_t_i128)]));
        }*/

        // for some very strange reason, they arent supported for i8 either, but that case
        // is easy to handle and we declare our own functions for that which just
        // zext to i16, use the i16 intrinsic, then trunc back to i8

        // these are declared in libintrinsics, see libintrinsics.ll
        ifn!("__nvvm_i8_addo", fn(t_i8, t_i8) -> t_i8_i1);
        ifn!("__nvvm_u8_addo", fn(t_i8, t_i8) -> t_i8_i1);
        ifn!("__nvvm_i8_subo", fn(t_i8, t_i8) -> t_i8_i1);
        ifn!("__nvvm_u8_subo", fn(t_i8, t_i8) -> t_i8_i1);
        ifn!("__nvvm_i8_mulo", fn(t_i8, t_i8) -> t_i8_i1);
        ifn!("__nvvm_u8_mulo", fn(t_i8, t_i8) -> t_i8_i1);

        // see comment in libintrinsics.ll
        // ifn!("__nvvm_i128_trap", fn(t_i128, t_i128) -> t_i128);

        ifn!("llvm.sadd.sat.i8", fn(t_i8, t_i8) -> t_i8);
        ifn!("llvm.sadd.sat.i16", fn(t_i16, t_i16) -> t_i16);
        ifn!("llvm.sadd.sat.i32", fn(t_i32, t_i32) -> t_i32);
        ifn!("llvm.sadd.sat.i64", fn(t_i64, t_i64) -> t_i64);

        ifn!("llvm.uadd.sat.i8", fn(t_i8, t_i8) -> t_i8);
        ifn!("llvm.uadd.sat.i16", fn(t_i16, t_i16) -> t_i16);
        ifn!("llvm.uadd.sat.i32", fn(t_i32, t_i32) -> t_i32);
        ifn!("llvm.uadd.sat.i64", fn(t_i64, t_i64) -> t_i64);

        ifn!("llvm.ssub.sat.i8", fn(t_i8, t_i8) -> t_i8);
        ifn!("llvm.ssub.sat.i16", fn(t_i16, t_i16) -> t_i16);
        ifn!("llvm.ssub.sat.i32", fn(t_i32, t_i32) -> t_i32);
        ifn!("llvm.ssub.sat.i64", fn(t_i64, t_i64) -> t_i64);

        ifn!("llvm.usub.sat.i8", fn(t_i8, t_i8) -> t_i8);
        ifn!("llvm.usub.sat.i16", fn(t_i16, t_i16) -> t_i16);
        ifn!("llvm.usub.sat.i32", fn(t_i32, t_i32) -> t_i32);
        ifn!("llvm.usub.sat.i64", fn(t_i64, t_i64) -> t_i64);

        ifn!("llvm.fshl.i8", fn(t_i8, t_i8, t_i8) -> t_i8);
        ifn!("llvm.fshl.i16", fn(t_i16, t_i16, t_i16) -> t_i16);
        ifn!("llvm.fshl.i32", fn(t_i32, t_i32, t_i32) -> t_i32);
        ifn!("llvm.fshl.i64", fn(t_i64, t_i64, t_i64) -> t_i64);

        ifn!("llvm.fshr.i8", fn(t_i8, t_i8, t_i8) -> t_i8);
        ifn!("llvm.fshr.i16", fn(t_i16, t_i16, t_i16) -> t_i16);
        ifn!("llvm.fshr.i32", fn(t_i32, t_i32, t_i32) -> t_i32);
        ifn!("llvm.fshr.i64", fn(t_i64, t_i64, t_i64) -> t_i64);

        ifn!("llvm.ctpop.i8", fn(t_i8) -> t_i8);
        ifn!("llvm.ctpop.i16", fn(t_i16) -> t_i16);
        ifn!("llvm.ctpop.i32", fn(t_i32) -> t_i32);
        ifn!("llvm.ctpop.i64", fn(t_i64) -> t_i64);

        ifn!("llvm.bitreverse.i8", fn(t_i8) -> t_i8);
        ifn!("llvm.bitreverse.i16", fn(t_i16) -> t_i16);
        ifn!("llvm.bitreverse.i32", fn(t_i32) -> t_i32);
        ifn!("llvm.bitreverse.i64", fn(t_i64) -> t_i64);

        ifn!("llvm.bswap.i16", fn(t_i16) -> t_i16);
        ifn!("llvm.bswap.i32", fn(t_i32) -> t_i32);
        ifn!("llvm.bswap.i64", fn(t_i64) -> t_i64);

        ifn!("llvm.ctlz.i8", fn(t_i8, i1) -> t_i8);
        ifn!("llvm.ctlz.i16", fn(t_i16, i1) -> t_i16);
        ifn!("llvm.ctlz.i32", fn(t_i32, i1) -> t_i32);
        ifn!("llvm.ctlz.i64", fn(t_i64, i1) -> t_i64);

        ifn!("llvm.cttz.i8", fn(t_i8, i1) -> t_i8);
        ifn!("llvm.cttz.i16", fn(t_i16, i1) -> t_i16);
        ifn!("llvm.cttz.i32", fn(t_i32, i1) -> t_i32);
        ifn!("llvm.cttz.i64", fn(t_i64, i1) -> t_i64);

        ifn!("llvm.lifetime.start.p0i8", fn(t_i64, i8p) -> void);
        ifn!("llvm.lifetime.end.p0i8", fn(t_i64, i8p) -> void);

        ifn!("llvm.expect.i1", fn(i1, i1) -> i1);
        ifn!("llvm.prefetch", fn(i8p, t_i32, t_i32, t_i32) -> void);

        // This isn't an "LLVM intrinsic", but LLVM's optimization passes
        // recognize it like one and we assume it exists in `core::slice::cmp`
        ifn!("memcmp", fn(i8p, i8p, t_isize) -> t_i32);

        ifn!("llvm.va_start", fn(i8p) -> void);
        ifn!("llvm.va_end", fn(i8p) -> void);
        ifn!("llvm.va_copy", fn(i8p, i8p) -> void);

        /* 
        if self.tcx.sess.opts.debuginfo != DebugInfo::None {
            ifn!("llvm.dbg.declare", fn(self.type_metadata(), self.type_metadata()) -> void);
            ifn!("llvm.dbg.value", fn(self.type_metadata(), t_i64, self.type_metadata()) -> void);
        }*/

        // misc syscalls, only the ones we use

        ifn!("vprintf", fn(i8p, voidp) -> t_i32);

        // so, nvvm, instead of allowing llvm math intrinsics and resolving them
        // to libdevice intrinsics, it forces us to explicitly use the libdevice
        // intrinsics and add libdevice as a module. so we need to completely
        // substitute the llvm intrinsics we would use, for the libdevice ones
        //
        // see https://docs.nvidia.com/cuda/pdf/libdevice-users-guide.pdf
        // and https://docs.nvidia.com/cuda/libdevice-users-guide/index.html
        // for docs on what these do

        // libdevice includes a lot of "exotic" intrinsics for common-ish formulas.
        // we might want to do a pass in the future to substitute common ops for
        // special libdevice intrinsics. We should also expose them as util traits
        // for f32 and f64 in cuda_std.

        ifn!("__nv_abs", fn(t_i32) -> t_i32);

        // f64 -> f64 intrinsics
        ifn!(
            "__nv_acos" |
            "__nv_acosh" |
            "__nv_asin" |
            "__nv_asinh" |
            "__nv_atan" |
            "__nv_atanh" |
            "__nv_cbrt" |
            "__nv_ceil" |
            "__nv_cos" |
            "__nv_cosh" |
            "__nv_cospi" |
            "__nv_drcp_rd" |
            "__nv_drcp_rn" |
            "__nv_drcp_ru" |
            "__nv_drcp_rz" |
            "__nv_dsqrt_rd" |
            "__nv_dsqrt_rn" |
            "__nv_dsqrt_ru" |
            "__nv_dsqrt_rz" |
            "__nv_erf" |
            "__nv_erfc" |
            "__nv_erfcinv" |
            "__nv_erfcx" |
            "__nv_erfinv" |
            "__nv_exp" |
            "__nv_exp10" |
            "__nv_exp2" |
            "__nv_expm1" |
            "__nv_fabs" |
            "__nv_floor" |
            "__nv_j0" |
            "__nv_j1" |
            "__nv_lgamma" |
            "__nv_log" |
            "__nv_log10" |
            "__nv_log1p" |
            "__nv_log2" |
            "__nv_logb" |
            "__nv_nearbyint" |
            "__nv_normcdf" |
            "__nv_normcdfinv" |
            "__nv_rcbrt" |
            "__nv_rint" |
            "__nv_round" |
            "__nv_rsqrt" |
            "__nv_sin" |
            "__nv_sinh" |
            "__nv_sinpi" |
            "__nv_sqrt" |
            "__nv_tan" |
            "__nv_tanh" |
            "__nv_tgamma" |
            "__nv_trunc" |
            "__nv_y0" |
            "__nv_y1",
            fn(t_f64) -> t_f64
        );

        // f32 -> f32 intrinsics
        ifn!(
            "__nv_acosf" |
            "__nv_acoshf" |
            "__nv_asinf" |
            "__nv_asinhf" |
            "__nv_atanf" |
            "__nv_atanhf" |
            "__nv_cbrtf" |
            "__nv_ceilf" |
            "__nv_cosf" |
            "__nv_coshf" |
            "__nv_cospif" |
            "__nv_erff" |
            "__nv_erfcf" |
            "__nv_erfcinvf" |
            "__nv_erfcxf" |
            "__nv_erfinvf" |
            "__nv_expf" |
            "__nv_exp10f" |
            "__nv_exp2f" |
            "__nv_expm1f" |
            "__nv_fabsf" |
            "__nv_floorf" |
            "__nv_j0f" |
            "__nv_j1f" |
            "__nv_lgammaf" |
            "__nv_logf" |
            "__nv_log10f" |
            "__nv_log1pf" |
            "__nv_log2f" |
            "__nv_logbf" |
            "__nv_nearbyintf" |
            "__nv_normcdff" |
            "__nv_normcdfinvf" |
            "__nv_rcbrtf" |
            "__nv_rintf" |
            "__nv_roundf" |
            "__nv_rsqrtf" |
            "__nv_sinf" |
            "__nv_sinhf" |
            "__nv_sinpif" |
            "__nv_sqrtf" |
            "__nv_tanf" |
            "__nv_tanhf" |
            "__nv_tgammaf" |
            "__nv_truncf" |
            "__nv_y0f" |
            "__nv_y1f",
            fn(t_f32) -> t_f32
        );

        // f64, f64 -> f64 intrinsics
        ifn!(
            
            "__nv_atan2" |
            "__nv_copysign" |
            "__nv_dadd_rd" |
            "__nv_dadd_rn" |
            "__nv_dadd_ru" |
            "__nv_dadd_rz" |
            "__nv_ddiv_rd" |
            "__nv_ddiv_rn" |
            "__nv_ddiv_ru" |
            "__nv_ddiv_rz" |
            "__nv_dmul_rd" |
            "__nv_dmul_rn" |
            "__nv_dmul_ru" |
            "__nv_dmul_rz" |
            "__nv_fdim" |
            "__nv_fmax" |
            "__nv_fmin" |
            "__nv_fmod" |
            "__nv_hypot" |
            "__nv_nextafter" |
            "__nv_pow" |
            "__nv_remainder",
            fn(t_f64, t_f64) -> t_f64
        );

        // f32, f32 -> f32 intrinsics
        ifn!(
            
            "__nv_atan2f" |
            "__nv_copysignf" |
            "__nv_fadd_rd" |
            "__nv_fadd_rn" |
            "__nv_fadd_ru" |
            "__nv_fadd_rz" |
            "__nv_fast_fdividef" |
            "__nv_fast_powf" |
            "__nv_fdimf" |
            "__nv_fdiv_rd" |
            "__nv_fdiv_rn" |
            "__nv_fdiv_ru" |
            "__nv_fdiv_rz" |
            "__nv_fmaxf" |
            "__nv_fminf" |
            "__nv_fmodf" |
            "__nv_fmul_rd" |
            "__nv_fmul_rn" |
            "__nv_fmul_ru" |
            "__nv_fmul_rz" |
            "__nv_fsub_rd" |
            "__nv_fsub_rn" |
            "__nv_fsub_ru" |
            "__nv_fsub_rz" |
            "__nv_hypotf" |
            "__nv_nextafterf" |
            "__nv_powf" |
            "__nv_remainderf",
            fn(t_f32, t_f32) -> t_f32
        );

        // other intrinsics

        ifn!(
            
            "__nv_powi",
            fn(t_f64, t_i32) -> t_f64
        );

        ifn!(
            
            "__nv_powif",
            fn(t_f32, t_i32) -> t_f32
        );

        ifn!(
            
            "__nv_fma",
            fn(t_f64, t_f64, t_f64) -> t_f64
        );

        ifn!(
            
            "__nv_fmaf",
            fn(t_f32, t_f32, t_f32) -> t_f32
        );

        ifn!(
            
            "__nv_yn",
            fn(t_i32, t_f64) -> t_f64
        );

        ifn!(
            
            "__nv_ynf",
            fn(t_i32, t_f32) -> t_f32
        );
    

    }
}
