use rustc_data_structures::fx::FxHashMap;
use rustc_middle::mir::{
    self, Body, ConstOperand, MirPass, Operand, TerminatorKind,
};
use rustc_middle::ty::{self, ConstKind, TyCtxt};
use rustc_span::def_id::DefId;
use rustc_hir::LangItem as LangItemVariant;
use rustc_hir::lang_items::LanguageItems;


pub struct KernelLangItemSwap {
    swap_map: FxHashMap<DefId, DefId>,
}

impl KernelLangItemSwap {
    pub fn new(tcx: TyCtxt<'_>) -> Self {
        let mut swap_map = FxHashMap::default();

        let items_to_swap_config: &[(LangItemVariant, fn(&LanguageItems) -> Option<DefId>)] = &[
            (LangItemVariant::PanicImpl, |li: &LanguageItems| li.kernel_panic_impl()),
            (LangItemVariant::PanicFmt,  |li: &LanguageItems| li.kernel_panic_fmt_impl()),
        ];

        let lang_items_instance = tcx.lang_items();

        for (std_item_variant, get_kernel_item_def_id_fn) in items_to_swap_config {
            if let Some(std_def_id) = lang_items_instance.get(*std_item_variant) {
                if let Some(kernel_def_id) = get_kernel_item_def_id_fn(lang_items_instance) {
                    eprintln!(
                        "KernelLangItemSwap: Configuring swap for {:?} ({:?}) to kernel version ({:?})",
                        std_item_variant, std_def_id, kernel_def_id
                    );
                    swap_map.insert(std_def_id, kernel_def_id);
                } else {
                    println!(
                        "KernelLangItemSwap: Kernel implementation for {:?} lang item not found.",
                        std_item_variant
                    );
                }
            } else {
                println!(
                    "KernelLangItemSwap: Standard {:?} lang item not found.",
                    std_item_variant
                );
            }
        }

        if swap_map.is_empty() {
            println!("KernelLangItemSwap: No lang item swaps were configured or successfully resolved.");
        }

        Self { swap_map }
    }
}

impl<'tcx> MirPass<'tcx> for KernelLangItemSwap {
    fn name(&self) -> &'static str {
        "KernelLangItemSwap"
    }

    fn is_enabled(&self, _sess: &rustc_session::Session) -> bool {
        !self.swap_map.is_empty()
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        if self.swap_map.is_empty() {
            return;
        }

        let current_fn_def_id = body.source.def_id();
        let fn_path_str = tcx.def_path_str(current_fn_def_id);

        println!(
            "KernelLangItemSwap: Processing function {} for lang item swaps",
            fn_path_str
        );

        for block in body.basic_blocks_mut() {
            let terminator = block.terminator_mut();

            if let TerminatorKind::Call { func, .. } =
                &mut terminator.kind
            {
                let callee_def_id_opt = match func {
                    Operand::Constant(box ConstOperand { const_, .. }) => {
                        match const_.ty().kind() {
                            ty::FnDef(def_id, _substs) => Some(*def_id),
                            _ => None,
                        }
                    }
                    _ => None,
                };

                if let Some(callee_def_id) = callee_def_id_opt {
                    if let Some(&kernel_target_def_id) = self.swap_map.get(&callee_def_id) {
                        let span = terminator.source_info.span;
                        let new_fn_ty = tcx.type_of(kernel_target_def_id).instantiate_identity();
                        let new_ty_const = ty::Const::new(
                            tcx,
                            ConstKind::Value(ty::ValTree::zst()),
                            new_fn_ty,
                        );
                        let new_mir_const = mir::Const::from_ty_const(new_ty_const, tcx);

                        *func = Operand::Constant(Box::new(ConstOperand {
                            span,
                            user_ty: None,
                            const_: new_mir_const,
                        }));
                        println!(
                            "KernelLangItemSwap: In {}, swapped call to lang item {:?} with kernel version {:?} ({})",
                            fn_path_str,
                            tcx.def_path_str(callee_def_id),
                            tcx.def_path_str(kernel_target_def_id),
                            tcx.item_name(callee_def_id).as_str()
                        );
                    }
                }
            }
        }
    }
}
