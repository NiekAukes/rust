use std::{cell::RefCell, rc::Rc};

use rustc_codegen_ssa::traits::{BackendTypes, MiscMethods};
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_middle::{mir::mono::CodegenUnit, ty::{layout::{HasParamEnv, HasTyCtxt}, Instance, ParamEnv, PolyExistentialTraitRef, Ty, TyCtxt}};
use rspirv::{spirv::Word, sr::instructions::ExtInst};
use rspirv::dr;
use rustc_session::Session;
use rustc_span::DUMMY_SP;
use rustc_target::{abi::{call::PassMode, AddressSpace, HasDataLayout, TargetDataLayout}, spec::{HasTargetSpec, Target}};
use crate::{builder_spirv::{BuilderSpirv, SpirvConst,SpirvValue}, spirv_type::{SpirvType, SpirvTypePrinter, TypeCache}, symbols::Symbols, };


pub struct CodegenCx<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub codegen_unit: &'tcx CodegenUnit<'tcx>,
    /// Spir-v module builder
    pub module_builder: BuilderSpirv<'tcx>,
    /// Map from MIR function to spir-v function ID
    pub instances: RefCell<FxHashMap<Instance<'tcx>, SpirvValue>>,
    /// Map from function ID to parameter list
    pub function_parameter_values: RefCell<FxHashMap<Word, Vec<SpirvValue>>>,
    pub type_cache: TypeCache<'tcx>,
    /// Cache generated vtables
    pub vtables: RefCell<FxHashMap<(Ty<'tcx>, Option<PolyExistentialTraitRef<'tcx>>), SpirvValue>>,
    pub ext_inst: RefCell<ExtInst>,
    /// Invalid SPIR-V IDs that should be stripped from the final binary,
    /// each with its own reason and span that should be used for reporting
    /// (in the event that the value is actually needed)
    //zombie_decorations:
    //    RefCell<FxHashMap<Word, (ZombieDecoration<'tcx>, Option<SrcLocDecoration<'tcx>>)>>,
    
    //pub instruction_table: InstructionTable,
    pub libm_intrinsics: RefCell<FxHashMap<Word, super::builder::libm_intrinsics::LibmIntrinsic>>,

    /// All `panic!(...)`s and builtin panics (from MIR `Assert`s) call into one
    /// of these lang items, which we always replace with an "abort".
    pub panic_entry_point_ids: RefCell<FxHashSet<Word>>,

    /// `core::fmt::Arguments::new_{v1,const}` instances (for Rust 2021 panics).
    pub fmt_args_new_fn_ids: RefCell<FxHashSet<Word>>,

    /// `core::fmt::rt::Argument::new_*::<T>` instances (for panics' `format_args!`),
    /// with their `T` type (i.e. of the value being formatted), and formatting
    /// "specifier" as a `char` (' ' for `Display`, `x` for `LowerHex`, etc.)
    pub fmt_rt_arg_new_fn_ids_to_ty_and_spec: RefCell<FxHashMap<Word, (Ty<'tcx>, char)>>,

    /// Intrinsic for loading a <T> from a &[u32]. The PassMode is the mode of the <T>.
    pub buffer_load_intrinsic_fn_id: RefCell<FxHashMap<Word, &'tcx PassMode>>,
    /// Intrinsic for storing a <T> into a &[u32]. The PassMode is the mode of the <T>.
    pub buffer_store_intrinsic_fn_id: RefCell<FxHashMap<Word, &'tcx PassMode>>,

    /// Some runtimes (e.g. intel-compute-runtime) disallow atomics on i8 and i16, even though it's allowed by the spec.
    /// This enables/disables them.
    pub i8_i16_atomics_allowed: bool,

    //pub codegen_args: CodegenArgs,

    // Information about the SPIR-V target.
    //pub target: SpirvTarget,

    pub sym: Symbols
}

impl BackendTypes for CodegenCx<'_> {
    type Value = SpirvValue;
    type Function = SpirvValue;
    type BasicBlock = Word;
    type Type = Word;
    type Funclet = ();
    // no debug info for now
    type DIScope = ();
    type DIVariable = ();
    type DILocation = ();
}

impl<'tcx> HasTyCtxt<'tcx> for CodegenCx<'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}

impl<'tcx> HasDataLayout for CodegenCx<'tcx> {
    fn data_layout(&self) -> &TargetDataLayout {
        &self.tcx.data_layout
    }
}

impl<'tcx> HasTargetSpec for CodegenCx<'tcx> {
    fn target_spec(&self) -> &Target {
        //&self.tcx.sess.target
        todo!()
    }
}

impl<'tcx> HasParamEnv<'tcx> for CodegenCx<'tcx> {
    fn param_env(&self) -> ParamEnv<'tcx> {
        ParamEnv::reveal_all()
    }
}

impl<'tcx> MiscMethods<'tcx> for CodegenCx<'tcx> {
    #[allow(clippy::type_complexity)]
    fn vtables(
        &self,
    ) -> &RefCell<FxHashMap<(Ty<'tcx>, Option<PolyExistentialTraitRef<'tcx>>), Self::Value>> {
        &self.vtables
    }

    fn check_overflow(&self) -> bool {
        self.tcx.sess.overflow_checks()
    }

    fn get_fn(&self, instance: Instance<'tcx>) -> Self::Function {
        self.get_fn_ext(instance)
    }

    // NOTE(eddyb) see the comment on `SpirvValueKind::FnAddr`, this should
    // be fixed upstream, so we never see any "function pointer" values being
    // created just to perform direct calls.
    fn get_fn_addr(&self, instance: Instance<'tcx>) -> Self::Value {
        let function = self.get_fn(instance);
        let span = self.tcx.def_span(instance.def_id());

        let ty = SpirvType::Pointer {
            pointee: function.ty,
        }
        .def(span, self);

        // Create these `OpUndef`s up front, instead of on-demand in `SpirvValue::def`,
        // because `SpirvValue::def` can't use `cx.emit()`.
        self.def_constant(ty, SpirvConst::ZombieUndefForFnAddr);

        todo!()
        /*SpirvValue {
            kind: SpirvValueKind::FnAddr {
                function: function.def_cx(self),
            },
            ty,
        }*/
    }

    fn eh_personality(&self) -> Self::Value {
        todo!()
    }

    fn sess(&self) -> &Session {
        self.tcx.sess
    }

    fn codegen_unit(&self) -> &'tcx CodegenUnit<'tcx> {
        self.codegen_unit
    }

    fn set_frame_pointer_type(&self, _llfn: Self::Function) {
        todo!()
    }

    fn apply_target_cpu_attr(&self, _llfn: Self::Function) {
        todo!()
    }

    fn declare_c_main(&self, _fn_type: Self::Type) -> Option<Self::Function> {
        todo!()
    }
}


impl<'tcx> CodegenCx<'tcx> {
    // dummy implementations
    pub fn emit_global(&self) -> std::cell::RefMut<'_, rspirv::dr::Builder> {
        todo!()
        /*self.builder.builder(BuilderCursor {
            function: None,
            block: None,
        })*/
    }

    #[track_caller]
    pub fn lookup_type(&self, ty: Word) -> SpirvType<'tcx> {
        self.type_cache.lookup(ty)
    }

    pub fn debug_type(&self, ty: Word) -> SpirvTypePrinter<'_, 'tcx> {
        self.lookup_type(ty).debug(ty, self)
    }

    pub fn type_ptr_to(&self, ty: Word) -> Word {
        SpirvType::Pointer { pointee: ty }.def(DUMMY_SP, self)
    }

    pub fn type_ptr_to_ext(&self, ty: Word, _address_space: AddressSpace) -> Word {
        SpirvType::Pointer { pointee: ty }.def(DUMMY_SP, self)
    }

    pub fn get_fn_ext(&self, instance: Instance<'tcx>) -> SpirvValue {
        todo!()
    }
}