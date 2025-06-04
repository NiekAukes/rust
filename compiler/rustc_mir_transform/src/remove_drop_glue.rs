use rustc_middle::mir::{
    BasicBlock, BasicBlockData, Body, MirPass, Statement, StatementKind, 
    Terminator, TerminatorKind
};
use rustc_middle::ty::TyCtxt;

pub struct RemoveDropGlue;

impl<'tcx> MirPass<'tcx> for RemoveDropGlue {
    fn name(&self) -> &'static str {
        "RemoveDropGlue"
    }

    fn is_enabled(&self, _sess: &rustc_session::Session) -> bool {
        true
    }

    fn run_pass(&self, tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
        let current_fn_def_id = body.source.def_id();
        let fn_path_str = tcx.def_path_str(current_fn_def_id);

        println!(
            "RemoveDropGlue: Processing function {} to remove drop glue",
            fn_path_str
        );

        self.remove_drop_statements_and_terminators(body, &fn_path_str);
        self.remove_cleanup_blocks(body, &fn_path_str);
    }
}

impl RemoveDropGlue {
    fn remove_drop_statements_and_terminators(&self, body: &mut Body<'_>, fn_path_str: &str) {
        for (block_idx, block_data) in body.basic_blocks_mut().iter_enumerated_mut() {
            block_data.statements.retain(|stmt| {
                match &stmt.kind {
                    StatementKind::StorageDead(_) => {
                        println!(
                            "RemoveDropGlue: In {}, removed StorageDead statement in block {:?}",
                            fn_path_str, block_idx
                        );
                        false
                    }
                    _ => true,
                }
            });

            match &block_data.terminator().kind {
                TerminatorKind::Drop { target, .. } => {
                    block_data.terminator_mut().kind = TerminatorKind::Goto { target: *target };
                    
                    println!(
                        "RemoveDropGlue: In {}, replaced Drop terminator with Goto in block {:?}",
                        fn_path_str, block_idx
                    );
                }
                _ => {
                }
            }
        }
    }

    fn remove_cleanup_blocks(&self, body: &mut Body<'_>, fn_path_str: &str) {
        let mut blocks_to_remove = Vec::new();

        for (block_idx, block_data) in body.basic_blocks.iter_enumerated() {
            if self.is_cleanup_block(block_data) {
                blocks_to_remove.push(block_idx);
                println!(
                    "RemoveDropGlue: In {}, identified cleanup block {:?} for removal",
                    fn_path_str, block_idx
                );
            }
        }

        for block_idx in blocks_to_remove {
            let empty_block = BasicBlockData {
                statements: vec![],
                terminator: Some(Terminator {
                    source_info: body.basic_blocks[block_idx].terminator().source_info,
                    kind: TerminatorKind::Unreachable,
                }),
                is_cleanup: false,
            };
            body.basic_blocks_mut()[block_idx] = empty_block;
        }
    }

    fn is_cleanup_block(&self, block_data: &BasicBlockData<'_>) -> bool {
        if block_data.is_cleanup {
            return true;
        }

        let only_cleanup_statements = block_data.statements.iter().all(|stmt| {
            matches!(
                stmt.kind,
                StatementKind::StorageDead(_) | 
                StatementKind::StorageLive(_) |
                StatementKind::Nop
            )
        });

        let cleanup_terminator = matches!(
            block_data.terminator().kind,
            TerminatorKind::Drop { .. } |
            TerminatorKind::UnwindResume |
            TerminatorKind::UnwindTerminate(_) |
            TerminatorKind::Unreachable
        );

        only_cleanup_statements && cleanup_terminator
    }
}