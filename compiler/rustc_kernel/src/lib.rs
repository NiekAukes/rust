use rustc_hir::def_id::DefId;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrFlags;
use rustc_middle::mir::interpret::ConstAllocation;
use rustc_middle::ty::{Instance, TyCtxt};
use rustc_middle::query::Providers;
use rustc_middle::bug;
use rustc_hir::def::DefKind;
use rustc_middle::mir::mono::{CodegenUnit, KernelMetaData, Linkage, MonoItem, MonoItemData, Visibility};
use rustc_middle::kernel::KernelCode;

mod kernel_embedder;

#[macro_use]
extern crate tracing;

pub fn is_kernel(tcx: TyCtxt, def_id: DefId) -> bool {
    // the def_id is a kernel if it is function-like and has a kernel flag
    if tcx.def_kind(def_id) != DefKind::Fn {
        return false;
    }

    let fn_attrs = tcx.codegen_fn_attrs(def_id).flags;
    fn_attrs.contains(CodegenFnAttrFlags::KERNEL)
}

pub fn build_cgu<'tcx>(tcx: TyCtxt<'tcx>, old_cgu: &'tcx CodegenUnit<'tcx>, size_estimate: usize) -> CodegenUnit<'tcx> {
    // create a new CGU with the same name as the old one
    // with only the entry def_id as a static instance
    let entry_def_id = old_cgu.kernel().unwrap().entry_def_id;
    let mono_item = MonoItem::Static(entry_def_id);
    let mono_item_data = MonoItemData {
        inlined: false,
        linkage: Linkage::LinkOnceAny,
        visibility: Visibility::Default,
        size_estimate: size_estimate,
    };

    let mut cgu = CodegenUnit::new(old_cgu.name(), None);
    cgu.items_mut().insert(mono_item, mono_item_data);
    cgu
}

pub fn compile_kernel_module_inner<'tcx>(tcx: TyCtxt<'tcx>, cgu: &'tcx CodegenUnit<'tcx>)
    -> &'tcx KernelCode<'tcx> {
    debug!("compiling kernel module {:?}", cgu.name());
    debug!("is kernel: {:?}", cgu.is_kernel());
    debug!("some kernel metadata: {:?}", cgu.kernel());
    let kernel_metadata = match cgu.kernel() {
        Some(k) => k,
        None => bug!("no kernel metadata found for {:?}", cgu.name()),
    };
    
    // TODO! compile the kernel module
    let code = b"1234";
    let constant = kernel_embedder::embed_kernel(tcx, code, kernel_metadata);
    let new_cgu = build_cgu(tcx, cgu, code.len());
    let new_cgu = tcx.arena.alloc(new_cgu);
    // compile the kernel module using processed_kernel_mir
    // and return a new modified CGU that compiles that processed kernel MIR
    //let ca = tcx.arena.alloc(ca);
    tcx.arena.alloc(KernelCode {
        const_body: constant,
        cgu: new_cgu,
        kernel_metadata: kernel_metadata,
    })
}

pub fn provide(providers: &mut Providers) {
    providers.compile_kernel_module = |tcx, module| {
        let cgu = tcx.kernel_unit(module);
        compile_kernel_module_inner(tcx, cgu)
    };

    providers.processed_kernel_mir = |tcx, def_id| {
        // get the cgu name
        let cgu_name = tcx.kernel_def_id_cgu_symbol(def_id);
        &tcx.compile_kernel_module(cgu_name).const_body
    };

    providers.is_kernel = is_kernel;
}
