use crate::ty::TyNVVM;

#[derive(Debug)]
pub struct FunctionNVVM<'m> {
    pub is_kernel: bool,
    pub name: String,
    pub ret: Option<TyNVVM<'m>>,
    pub args: Vec<TyNVVM<'m>>,
}


impl PartialEq for FunctionNVVM<'_> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'m> FunctionNVVM<'m> {
    pub fn new(name: String, 
        is_kernel: bool, 
        ret: Option<TyNVVM<'m>>,
        args: Vec<TyNVVM<'m>>) -> Self {
        Self {
            is_kernel,
            name,
            ret,
            args,
        }
    }
}