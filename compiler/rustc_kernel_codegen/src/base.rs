use rustc_codegen_ssa::mono_item::MonoItemExt;
use rustc_middle::{mir::mono::CodegenUnit, ty::TyCtxt};

use crate::arena;
use crate::{codegen_cx::CodegenCx, ModuleNVVM};
use crate::builder::Builder;

pub fn module_codegen<'m, 'tcx>(
    tcx: TyCtxt<'tcx>, 
    cgu: &'tcx CodegenUnit<'tcx>)
    -> Vec<u32>
 {
    let arena = arena::Arena::default();
    let mut module = ModuleNVVM::new(&arena);
    let cx = CodegenCx::new(tcx, module);
    let mono_items = cgu.items_in_deterministic_order(tcx);
    for &(mono_item, data) in &mono_items {
        println!("predefining {:?}", mono_item);
        mono_item.predefine::<Builder<'_, '_, '_>>(&cx, data.linkage, data.visibility);
    }

    // ... and now that we have everything pre-defined, fill out those definitions.
    for &(mono_item, _) in &mono_items {
        println!("defining {:?}", mono_item);
        mono_item.define::<Builder<'_, '_, '_>>(&cx);
    }

    // Run replace-all-uses-with for statics that need it. This must
    // happen after the llvm.used variables are created.
    /* 
    for &(old_g, new_g) in cx.statics_to_rauw().borrow().iter() {
        unsafe {
            llvm::LLVMReplaceAllUsesWith(old_g, new_g);
            llvm::LLVMDeleteGlobal(old_g);
        }
    }*/

    let m = cx.finalize();
    m.assemble()
}