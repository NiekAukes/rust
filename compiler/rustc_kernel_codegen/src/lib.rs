#![feature(extern_types)]
#![allow(unused)]

use rustc_middle::query::Providers;
mod builder;
mod codegen_cx;
mod llvm;

pub fn provide(providers: &mut Providers) {
    
}
