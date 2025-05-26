use std::cell::UnsafeCell;

use rustc_data_structures::fx::FxHashMap;
use rustc_middle::bug;

use crate::module::{Assemble, ModuleNVVM};
use crate::ty::TypeNVVM;
use crate::value::{Val, ValueNVVM};
use crate::{basic_block::BasicBlock, ty::TyNVVM};

#[derive(Debug)]
pub struct FunctionNVVM<'m> {
    pub is_kernel: bool,
    pub name: String,
    pub ret: TyNVVM<'m>,
    pub args: Vec<Val<'m>>,
    pub ty: TyNVVM<'m>,
    pub fnval: Option<Val<'m>>,
    basic_blocks: UnsafeCell<Vec<&'m BasicBlock<'m>>>,
    val_labels: UnsafeCell<FxHashMap<Val<'m>, String>>,
    counter: UnsafeCell<usize>,
    has_panic_block: UnsafeCell<bool>,
}

impl PartialEq for FunctionNVVM<'_> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'m> FunctionNVVM<'m> {
    pub fn new(
        name: String,
        is_kernel: bool,
        ret: TyNVVM<'m>,
        args: Vec<Val<'m>>,
        ty: TyNVVM<'m>,
    ) -> Self {
        Self {
            is_kernel,
            name,
            ret,
            args,
            ty,
            fnval: None,
            basic_blocks: UnsafeCell::new(Vec::new()),
            val_labels: UnsafeCell::new(FxHashMap::default()),
            counter: UnsafeCell::new(0),
            has_panic_block: UnsafeCell::new(false),
        }
    }

    pub fn new_inferred(
        name: String,
        ret: TyNVVM<'m>,
        args: Vec<Val<'m>>,
        module: &mut ModuleNVVM<'m>,
    ) -> Self {
        let ty = module.ty_from_type(TypeNVVM::Fn(
            args.iter()
                .map(|a| {
                    // all values should be params
                    if let ValueNVVM::Param { ty, .. } = a.0 {
                        *ty
                    } else {
                        panic!("Function arguments should be params");
                    }
                })
                .collect(),
            ret,
        ));
        Self {
            is_kernel: false,
            name,
            ret,
            args,
            ty,
            fnval: None,
            basic_blocks: UnsafeCell::new(Vec::new()),
            val_labels: UnsafeCell::new(FxHashMap::default()),
            counter: UnsafeCell::new(0),
            has_panic_block: UnsafeCell::new(false),
        }
    }

    pub fn add_basic_block(&self, bb: &'m BasicBlock<'m>) {
        unsafe {
            if (bb.name == "panic") {
                *self.has_panic_block.get() = true;
            } else {
                (*self.basic_blocks.get()).push(bb);
            }
        }
    }

    pub fn is_defined(&self) -> bool {
        unsafe { !(*self.basic_blocks.get()).is_empty() }
    }

    pub fn label_of_val(&self, val: Val<'m>) -> Option<String> {
        unsafe {
            if let Some(label) = (*self.val_labels.get()).get(&val) {
                Some(label.clone())
            } else {
                None
            }
        }
    }

    pub fn assign_label_to_val(&self, val: Val<'m>) -> String {
        unsafe {
            if let Some(label) = (*self.val_labels.get()).get(&val) {
                bug!("Value already has label: {:?}", label);
            } else {
                let label = format!("%{}", *self.counter.get());
                (*self.val_labels.get()).insert(val, label.clone());
                *self.counter.get() += 1;
                label
            }
        }
    }

    pub fn create_val_label(&self, val: Val<'m>, name: String) {
        unsafe {
            (*self.val_labels.get()).insert(val, name);
        }
    }

    pub fn iter_basic_blocks(&self) -> impl Iterator<Item = &&'m BasicBlock<'m>> {
        unsafe {
            let bbs = &*self.basic_blocks.get();
            bbs.iter()
        }
    }

    pub fn assemble(&self, module: &mut ModuleNVVM<'m>) -> String {
        let mut s = format!("define ");
        // return type
        s.push_str(&format!("{} ", self.ret.assemble(module)));
        // function name
        s.push_str(&format!("@{}(", self.name));

        // arguments
        for (i, arg) in self.args.iter().enumerate() {
            let arg_inner = arg.0;
            if let ValueNVVM::Param { .. } = arg_inner {
                if i != 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!("{}", arg_inner.assemble(module, Some(&self), arg)));
            }
        }
        s.push_str(") {\n");

        // basic blocks
        for bb in unsafe { &*self.basic_blocks.get() } {
            s.push_str(&format!("{}:\n", bb.name));
            // if bb.name == "panic" {
            //     // custom handle panic blocks, we need to do this due to some
            //     // limitations in the current implementation
            //     // the normal panic handler expects panic functions to be defined
            //     // but they aren't in the kernel, and we don't want to define them

            //     // so, we just skip the panic block skip the entire function
            //     // we do this by calling the llvm.trap intrinsic
            //     s.push_str("  call void @llvm.trap()\n");
            //     module.use_intrinsic("llvm.trap");
            //     //println!("using intrinsic llvm.trap");
            //     s.push_str("  unreachable\n");
            //     continue;
            // }
            for instr in bb.instrs() {
                let instr_inner = instr.0;
                s.push_str(&format!("  {}\n", instr_inner.assemble(module, Some(&self), &instr)));
            }
        }

        unsafe {
            if *self.has_panic_block.get() {
                // custom handle panic blocks, we need to do this due to some
                // limitations in the current implementation
                // the normal panic handler expects panic functions to be defined
                // but they aren't in the kernel, and we don't want to define them

                // so, we just skip the panic block skip the entire function
                // we do this by calling the llvm.trap intrinsic
                s.push_str("panic:\n");
                s.push_str("  call void @llvm.trap()\n");
                module.use_intrinsic("llvm.trap");
                //println!("using intrinsic llvm.trap");
                s.push_str("  unreachable\n");
            }
        }

        s.push_str("}\n");
        s
    }

    pub fn define(&self, module: &mut ModuleNVVM<'m>) -> String {
        let mut s = format!("declare ");
        // return type
        s.push_str(&format!("{} ", self.ret.assemble(module)));
        // function name
        s.push_str(&format!("@{}(", self.name));

        // arguments
        for (i, arg) in self.args.iter().enumerate() {
            let arg_inner = arg.0;
            if let ValueNVVM::Param { .. } = arg_inner {
                if i != 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!("{}", arg_inner.assemble(module, Some(&self), arg)));
            }
        }
        s.push_str(");\n");
        s
    }
}
