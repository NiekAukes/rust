use rustc_codegen_ssa::mono_item::MonoItemExt;
use rustc_middle::mir::mono::CodegenUnit;
use rustc_middle::ty::TyCtxt;

use crate::arena;
use crate::builder::Builder;
use crate::codegen_cx::CodegenCx;
use crate::module::{ModuleNVVM, assemble};

pub fn module_codegen_hack<'tcx>(
    tcx: TyCtxt<'tcx>,
    cgu: &'tcx CodegenUnit<'tcx>,
) -> (&'tcx str, Vec<u8>) {
    // hack for now
    // return a premade module based on the name of the kernel
    let mono_items = cgu.items_in_deterministic_order(tcx);
    let name = {
        let mut name = None;
        for &(mono_item, _) in &mono_items {
            if tcx.is_kernel(mono_item.def_id()) {
                name = Some(mono_item.symbol_name(tcx).name);
            }
        }
        name.expect("no kernel found")
    };

    panic!("kernel not found")
}

pub fn module_codegen<'tcx>(tcx: TyCtxt<'tcx>, cgu: &'tcx CodegenUnit<'tcx>) -> (String, String) {
    let arena = arena::Arena::default();
    let mut module = ModuleNVVM::new(&arena);
    let mut cx = CodegenCx::new(tcx, module);
    let mono_items = cgu.items_in_deterministic_order(tcx);

    let cgu_ns = cgu.name();
    let cgu_name = cgu_ns.as_str();

    cx.build_intrinsics();
    cx.build_kernel_allocator_shims();

    for &(mono_item, data) in &mono_items {
        println!("predefining {:?}", mono_item);
        mono_item.predefine::<Builder<'_, '_, '_>>(
            &mut cx,
            cgu_name,
            data.linkage,
            data.visibility,
        );
    }

    // ... and now that we have everything pre-defined, fill out those definitions.
    for &(mono_item, itemdata) in &mono_items {
        //println!("defining {:?}", mono_item);
        mono_item.define::<Builder<'_, '_, '_>>(&mut cx, cgu_name, itemdata);
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

    let mut m = cx.finalize();

    let name = {
        let mut name = None;
        for &(mono_item, _) in &mono_items {
            if tcx.is_kernel(mono_item.def_id()) {
                name = Some(mono_item.symbol_name(tcx).name);
            }
        }
        name.expect("no kernel found")
    };

    (name.to_string(), assemble(&mut m))
}
