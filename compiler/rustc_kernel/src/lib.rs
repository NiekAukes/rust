use rustc_middle::ty::TyCtxt;
use rustc_middle::query::Providers;

mod kernel_embedder;

#[macro_use]
extern crate tracing;

pub fn provide(providers: &mut Providers) {
    providers.compile_kernel_module = |tcx, module| {
        let cgu = tcx.kernel_unit(module);
        let kernel_metadata = cgu.kernel().unwrap_or(bug!("no kernel metadata for {:?}", module));
        let _ = kernel_embedder::embed_kernel(tcx, &[], kernel_metadata);
        // tcx.arena.alloc
        todo!()
    };
}
