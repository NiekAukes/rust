use rustc_middle::ty::TyCtxt;
use rustc_middle::query::Providers;
use rustc_middle::bug;

mod kernel_embedder;

#[macro_use]
extern crate tracing;

pub fn provide(providers: &mut Providers) {
    providers.compile_kernel_module = |tcx, module| {
        let cgu = tcx.kernel_unit(module);
        let kernel_metadata = cgu.kernel().unwrap_or(bug!("no kernel metadata for {:?}", module));

        // TODO! compile the kernel module

        let _ = kernel_embedder::embed_kernel(tcx, &[], kernel_metadata);
        // compile the kernel module using processed_kernel_mir
        // and return a new modified CGU that compiles that processed kernel MIR

        todo!()        
    };

    providers.processed_kernel_mir = |tcx, def_id| {
        // get the cgu name
        let cgu_name = tcx.kernel_def_id_cgu_symbol(def_id);
        tcx.compile_kernel_module(cgu_name).1
    };
}
