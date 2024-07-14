use std::cell::UnsafeCell;

use crate::{function::FunctionNVVM, value::Val};

#[derive(Debug)]
pub struct BasicBlock<'m> {
    pub func: &'m FunctionNVVM<'m>,
    pub name: String,
    instrs: UnsafeCell<Vec<Val<'m>>>,
}

impl<'m> BasicBlock<'m> {
    pub fn new(func: &'m FunctionNVVM<'m>, name: &str) -> Self {
        Self {
            func,
            name: name.to_string(),
            instrs: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn add_instr(&self, instr: Val<'m>) {
        unsafe {
            (*self.instrs.get()).push(instr);
        }
    }

    pub fn instrs(&self) -> Vec<Val<'m>> {
        unsafe { (*self.instrs.get()).clone() }
    }
}