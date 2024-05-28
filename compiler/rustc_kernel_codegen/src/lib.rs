#![feature(lazy_cell)]
#![feature(assert_matches)]

mod builder_spirv;
mod codegen_cx;
mod spirv_type;
mod abi;
mod target_feature;
mod target;
mod symbols;
mod builder;
mod custom_decorations;


macro_rules! assert_ty_eq {
    ($codegen_cx:expr, $left:expr, $right:expr) => {
        assert!(
            $left == $right,
            "Expected types to be equal:\n{}\n==\n{}",
            $codegen_cx.debug_type($left),
            $codegen_cx.debug_type($right)
        )
    };
}