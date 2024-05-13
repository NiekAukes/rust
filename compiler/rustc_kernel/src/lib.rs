use rustc_middle::ty::TyCtxt;
use rustc_middle::query::Providers;

mod kernel_embedder;

#[macro_use]
extern crate tracing;

pub fn provide(providers: &mut Providers) {
    providers.compile_kernel_module = |tcx, module| {
        //let mut codegen_backend = tcx.codegen_backend();
        //codegen_backend.compile_kernel_module(tcx, module)
        let _ = kernel_embedder::embed_kernel(tcx, &[]);
        // tcx.arena.alloc
        todo!()
    };
}
