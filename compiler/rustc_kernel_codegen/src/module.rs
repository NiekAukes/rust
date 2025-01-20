use std::iter::Map;

use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_hir::def_id::DefId;
use rustc_middle::{mir::interpret::AllocId, ty::Ty};

use crate::{function::FunctionNVVM, intrinsics::declare_intrinsics, ty::{TyNVVM, TypeHints, TypeNVVM}, value::{Val, ValueNVVM}, Arena, Global, GlobalNVVM};

pub trait Assemble<'m> {
    fn assemble(&self, module: &mut ModuleNVVM<'m>) -> String;
}

pub struct ModuleNVVM<'m> {
    pub functions: FxHashMap<DefId, &'m FunctionNVVM<'m>>,
    pub defrefs: FxHashMap<DefId, Val<'m>>,

    declared_intrinsics: FxHashSet<String>,
    pub intrinsics: FxHashMap<String, Val<'m>>,

    pub globals: FxHashMap<DefId, Global<'m>>,
    pub allocs: FxHashMap<Global<'m>, Val<'m>>,

    pub types: Vec<TyNVVM<'m>>,
    pub values: Vec<Val<'m>>,
    pub valtypes: FxHashMap<Val<'m>, TyNVVM<'m>>,

    pub metadata: Metadata<'m>,
    pub arena: &'m Arena<'m>,


    // internal state for assembling the module
    pub vallabels: FxHashMap<Val<'m>, String>,
    pub tylabels: FxHashMap<TyNVVM<'m>, String>,
    counter: usize,
}


impl<'m> ModuleNVVM<'m> {
    pub fn new(
        arena: &'m Arena<'m>,
    ) -> Self {
        let mut s = Self {
            functions: FxHashMap::default(),
            defrefs: FxHashMap::default(),
            declared_intrinsics: FxHashSet::default(),
            intrinsics: FxHashMap::default(),
            globals: FxHashMap::default(),
            allocs: FxHashMap::default(),
            metadata: Metadata {
                version: (1, 0),
                kernel: None,
            },
            types: Vec::new(),
            values: Vec::new(),
            valtypes: FxHashMap::default(),

            arena,

            vallabels: FxHashMap::default(),
            tylabels: FxHashMap::default(),
            counter: 0,
        };

        declare_intrinsics(s)
    }

    pub fn add_allocation(&mut self, alloc: GlobalNVVM<'m>) -> Val<'m> {
        // amend alloc with a new unique name
        let mut alloc = alloc;
        alloc.name = format!("global{}", self.allocs.len());

        let alloc = self.arena.dropless.alloc(alloc);
        
        // create a value for the allocation
        let ty_u8 = self.ty_from_type(TypeNVVM::I(8));
        let ty = if let Some(ref d) = alloc.data {
             self.ty_from_type(TypeNVVM::Array(ty_u8, d.len()))
        } else {
            self.ty_from_type(TypeNVVM::Pointer(ty_u8))
        };
        let ty_ptr = self.ty_from_type(TypeNVVM::Pointer(ty));
        let value = self.create_val(ValueNVVM::Global(alloc), Some(ty_ptr));

        let global = Global::new_unchecked(alloc);
        
        self.allocs.insert(global, value);
        value
    }
    
    pub fn add_function(&mut self, def_id: DefId, function: FunctionNVVM<'m>) {
        let function = self.arena.dropless.alloc(function);
        self.functions.insert(def_id, function);
    }

    pub fn ty_from_type(&mut self, ty: TypeNVVM<'m>) -> TyNVVM<'m> {
        // check if the type already exists
        for t in &self.types {
            if **t == ty {
                return *t;
            }
        }
        let alloc_type = self.arena.dropless.alloc(ty);
        let ty = TyNVVM::new_unchecked(alloc_type);
        self.types.push(ty);
        ty
    }

    pub fn create_val(&mut self, value: ValueNVVM<'m>, ty: Option<TyNVVM<'m>>) -> Val<'m> {
        // if the value is an instruction, always create a new value
        /*if let ValueNVVM::Instr(_) = value {
            let value = self.arena.dropless.alloc(value);
            return Val::new_unchecked(value);
        }*/

        // check if the value already exists
        /*for v in &self.values {
            if **v == value {
                return *v;
            }
        }*/

        //

        let alloc_value = self.arena.dropless.alloc(value);
        let value = Val::new_unchecked(alloc_value);

        // if the value has an associated type, add it to the valtypes map
        if let Some(ty) = ty {
            self.valtypes.insert(value, ty);
        }

        self.values.push(value);
        value
    }

    pub fn set_valtype(&mut self, value: Val<'m>, ty: TyNVVM<'m>) {
        self.valtypes.insert(value, ty);
    }

    pub fn label_of_val(&mut self, value: Val<'m>) -> &str {
        let l = self.counter;
        self.vallabels.entry(value).or_insert_with(|| { self.counter += 1; format!("%{}", l) })
    }

    pub fn create_val_label(&mut self, value: Val<'m>, name: String) {
        self.vallabels.insert(value, name);
    }

    pub fn create_ty_label(&mut self, ty: TyNVVM<'m>) -> &str {
        let l = self.counter;
        self.tylabels.entry(ty).or_insert_with(|| { self.counter += 1; format!("%{}", l) })
    }

    pub fn get_label_of_ty(&self, ty: TyNVVM<'m>) -> Option<&String> {
        self.tylabels.get(&ty)
    }

    /*pub fn create_typehint(&mut self, value: Val<'m>, size: usize) {
        self.valtypehints.insert(value, TypeHints::new(size));
    }

    pub fn add_struct_hint(&mut self, value: Val<'m>, offset: usize, size: usize) {
        if let Some(hint) = self.valtypehints.get_mut(&value) {
            hint.struct_hint(offset, size);
        }
    }

    pub fn add_type_hint(&mut self, value: Val<'m>, offset: usize, size: usize, ty: TyNVVM<'m>) {
        if let Some(hint) = self.valtypehints.get_mut(&value) {
            hint.layout_hint(offset, size, ty);
        }
    }*/

    pub fn get_intrinsic(&self, name: &str)-> Option<Val<'m>> {
        self.intrinsics.get(name).copied()
    }

    pub fn use_intrinsic(&mut self, name: &str) {
        self.declared_intrinsics.insert(name.to_string());
    }
}


pub fn assemble<'m>(module: &mut ModuleNVVM<'m>) -> String {
    let mut s = format!("; NVVM IR version {}\n", module.metadata.version.0);
    s.push_str("target datalayout = \"e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-v16:16:16-v32:32:32-v64:64:64-v128:128:128-n16:32:64\"\n");
    s.push_str("target triple = \"nvptx64-nvidia-cuda\"\n\n");
    
    let mut kernel = None;


    // define the types used in the module
    let tys = module.types.clone();
    /*for ty in tys {
        // if the type is not a struct, skip
        if let TypeNVVM::Struct(_) = *ty {
            let ty_str = ty.assemble(module);
            let label = module.create_ty_label(ty);
            s.push_str(&format!("{} = type {}\n", label, ty_str));
        }
    }
    s.push_str("\n");*/

    // define the globals used in the module
    let globals = module.globals.clone();
    for (_, global) in globals.iter() {
        todo!()
    }
    let allocs = module.allocs.clone();
    for (global, val) in allocs.iter() {
        s.push_str(&global.0.assemble(module));
    }
    s.push_str("\n");

    let fns = module.functions.clone();
    // define or declare the functions used in the module
    for (def_id, function) in fns.iter() {
        if !function.is_defined() {
            s.push_str(&function.define(module));
            s.push_str("\n\n");
        } else {
            s.push_str(&function.assemble(module));
            s.push_str("\n\n");
            if function.is_kernel {
                kernel = Some(function);
            }
        }
    }


    //define the intrinsics used in the module
    let declared_intrinsics = module.declared_intrinsics.clone();
    for name in declared_intrinsics {
        let val = module.get_intrinsic(&name).unwrap();
        let ty = *module.valtypes.get(&val).unwrap();
        let (args, ret) = match ty.0 {
            TypeNVVM::Fn(args, ty) => (args, ty),
            _ => panic!("Intrinsic should have function type"),
        };
        let ret_str = ret.assemble(module);
        let label = module.label_of_val(val);

        // TODO: add the arguments
        s.push_str(&format!("declare {} @{}()\n", ret_str, name));
    }
    s.push_str("\n");

    /*
    !nvvm.annotations = !{!1}
    !1 = !{void (i32*)* @simple, !"kernel", i32 1}

    !nvvmir.version = !{!2}
    !2 = !{i32 2, i32 0, i32 3, i32 1}
     */

    if let Some(kernel) = kernel {
        let kernel_name = kernel.name.clone();
        s.push_str(&format!("!nvvm.annotations = !{{!1}}\n"));
        s.push_str(&format!("!1 = !{{{}* @{}, !\"kernel\", i32 1}}\n", kernel.ty.assemble(module), kernel_name));
        
        s.push_str(&format!("!nvvmir.version = !{{!2}}\n"));
        s.push_str(&format!("!2 = !{{i32 {}, i32 {}}}\n", 2, 0));
    }
    s.push_str("\n");

    s
}


#[derive(Debug)]
pub struct Metadata<'m> {
    version: (u8, u8),
    kernel: Option<&'m FunctionNVVM<'m>>,
}


impl PartialEq for Metadata<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl<'m> Metadata<'m> {
    pub fn new(version: (u8, u8)) -> Self {
        Self {
            version,
            kernel: None,
        }
    }

    pub fn set_kernel(&mut self, kernel: &'m FunctionNVVM<'m>) {
        self.kernel = Some(kernel);
    }
}