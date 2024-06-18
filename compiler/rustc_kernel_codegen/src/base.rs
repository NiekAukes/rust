use rustc_codegen_ssa::mono_item::MonoItemExt;
use rustc_middle::{mir::mono::CodegenUnit, ty::TyCtxt};

use crate::arena;
use crate::{codegen_cx::CodegenCx, module::ModuleNVVM};
use crate::builder::Builder;

pub fn module_codegen_hack<'tcx>(
    tcx: TyCtxt<'tcx>, 
    cgu: &'tcx CodegenUnit<'tcx>)
    -> (&'tcx str, Vec<u8>)
 {
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
    println!("kernel name: {}", name);
    if name.contains("gpu64") {
        return ("simple", include_bytes!("GPU64.ll").to_vec());
    }
    
    panic!("kernel not found")
 }

pub fn module_codegen<'tcx>(
    tcx: TyCtxt<'tcx>, 
    cgu: &'tcx CodegenUnit<'tcx>)
    -> (String, Vec<u32>)
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

    let name = {
        let mut name = None;
        for &(mono_item, _) in &mono_items {
            if tcx.is_kernel(mono_item.def_id()) {
                name = Some(mono_item.symbol_name(tcx).name);
                
            }
        }
        name.expect("no kernel found")
    };



    (name.to_string(), m.assemble())
}