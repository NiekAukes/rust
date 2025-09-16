use core::panic;

use rustc_ast::{InlineAsmOperand, InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_codegen_ssa::{
    mir::operand::OperandValue,
    traits::{
        AsmBuilderMethods, BackendTypes, BaseTypeMethods, BuilderMethods, ConstMethods,
        InlineAsmOperandRef, MiscMethods,
    },
};
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::ty::layout::{HasTyCtxt, TyAndLayout};
use rustc_target::{
    abi::{Abi, Integer, Primitive, Scalar},
    asm::{
        AArch64InlineAsmReg, AArch64InlineAsmRegClass, ArmInlineAsmReg, ArmInlineAsmRegClass,
        AvrInlineAsmRegClass, BpfInlineAsmRegClass, CSKYInlineAsmRegClass,
        HexagonInlineAsmRegClass, InlineAsmArch, InlineAsmReg, InlineAsmRegClass,
        InlineAsmRegOrRegClass, LoongArchInlineAsmRegClass, M68kInlineAsmRegClass,
        MipsInlineAsmRegClass, Msp430InlineAsmRegClass, NvptxInlineAsmRegClass,
        PowerPCInlineAsmRegClass, RiscVInlineAsmRegClass, S390xInlineAsmRegClass,
        SpirVInlineAsmRegClass, WasmInlineAsmRegClass, X86InlineAsmReg, X86InlineAsmRegClass,
    },
};

use crate::{
    basic_block::BasicBlock,
    codegen_cx::CodegenCx,
    ty::TyNVVM,
    value::{InlineAsmNVVMOperand, Instruction, Val, ValueNVVM},
};

use super::Builder;

impl<'tcx> AsmBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn codegen_inline_asm(
        &mut self,
        template: &[rustc_ast::InlineAsmTemplatePiece],
        operands: &[InlineAsmOperandRef<'tcx, Self>],
        options: rustc_ast::InlineAsmOptions,
        line_spans: &[rustc_span::Span],
        instance: rustc_middle::ty::Instance<'_>,
        dest: Option<Self::BasicBlock>,
        catch_funclet: Option<(Self::BasicBlock, Option<&Self::Funclet>)>,
    ) {
        // COPIED FROM HERE rustc_codegen_llvm/asm.rs
        let asm_arch = self.tcx().sess.asm_arch.unwrap();

        // Collect the types of output operands
        let mut constraints = vec![];
        let mut clobbers = vec![];
        let mut output_types = vec![];
        let mut op_idx = FxHashMap::default();
        let mut clobbered_x87 = false;
        for (idx, op) in operands.iter().enumerate() {
            match *op {
                InlineAsmOperandRef::Out { reg, late, place } => {
                    let is_target_supported = |reg_class: InlineAsmRegClass| {
                        for &(_, feature) in reg_class.supported_types(asm_arch) {
                            if let Some(feature) = feature {
                                if self
                                    .tcx()
                                    .asm_target_features(instance.def_id())
                                    .contains(&feature)
                                {
                                    return true;
                                }
                            } else {
                                // Register class is unconditionally supported
                                return true;
                            }
                        }
                        false
                    };

                    let mut layout = None;
                    let ty = if let Some(ref place) = place {
                        layout = Some(&place.layout);
                        llvm_fixup_output_type(self.cx(), reg.reg_class(), &place.layout)
                    } else if matches!(
                        reg.reg_class(),
                        InlineAsmRegClass::X86(
                            X86InlineAsmRegClass::mmx_reg | X86InlineAsmRegClass::x87_reg
                        )
                    ) {
                        // Special handling for x87/mmx registers: we always
                        // clobber the whole set if one register is marked as
                        // clobbered. This is due to the way LLVM handles the
                        // FP stack in inline assembly.
                        if !clobbered_x87 {
                            clobbered_x87 = true;
                            clobbers.push("~{st}".to_string());
                            for i in 1..=7 {
                                clobbers.push(format!("~{{st({})}}", i));
                            }
                        }
                        continue;
                    } else if !is_target_supported(reg.reg_class())
                        || reg.reg_class().is_clobber_only(asm_arch)
                    {
                        // We turn discarded outputs into clobber constraints
                        // if the target feature needed by the register class is
                        // disabled. This is necessary otherwise LLVM will try
                        // to actually allocate a register for the dummy output.
                        assert!(matches!(reg, InlineAsmRegOrRegClass::Reg(_)));
                        clobbers.push(format!("~{}", reg_to_llvm(reg, None)));
                        continue;
                    } else {
                        // If the output is discarded, we don't really care what
                        // type is used. We're just using this to tell LLVM to
                        // reserve the register.
                        dummy_output_type(self.cx(), reg.reg_class())
                    };
                    output_types.push(ty);
                    op_idx.insert(idx, constraints.len());
                    // nvvm does not support early clobber
                    let prefix = "="; //if late { "=" } else { "=&" };
                    constraints.push(format!("{}{}", prefix, reg_to_llvm(reg, layout)));
                }
                InlineAsmOperandRef::InOut { reg, late, in_value, out_place } => {
                    let layout = if let Some(ref out_place) = out_place {
                        &out_place.layout
                    } else {
                        // LLVM required tied operands to have the same type,
                        // so we just use the type of the input.
                        &in_value.layout
                    };
                    let ty = llvm_fixup_output_type(self.cx(), reg.reg_class(), layout);
                    output_types.push(ty);
                    op_idx.insert(idx, constraints.len());
                    let prefix = if late { "=" } else { "=&" };
                    constraints.push(format!("{}{}", prefix, reg_to_llvm(reg, Some(layout))));
                }
                _ => {}
            }
        }

        // Collect input operands
        let mut inputs = vec![];
        for (idx, op) in operands.iter().enumerate() {
            match *op {
                InlineAsmOperandRef::In { reg, value } => {
                    let llval =
                        llvm_fixup_input(self, value.immediate(), reg.reg_class(), &value.layout);
                    inputs.push(llval);
                    op_idx.insert(idx, constraints.len());
                    constraints.push(reg_to_llvm(reg, Some(&value.layout)));
                }
                InlineAsmOperandRef::InOut { reg, late, in_value, out_place: _ } => {
                    let value = llvm_fixup_input(
                        self,
                        in_value.immediate(),
                        reg.reg_class(),
                        &in_value.layout,
                    );
                    inputs.push(value);

                    // In the case of fixed registers, we have the choice of
                    // either using a tied operand or duplicating the constraint.
                    // We prefer the latter because it matches the behavior of
                    // Clang.
                    if late && matches!(reg, InlineAsmRegOrRegClass::Reg(_)) {
                        constraints.push(reg_to_llvm(reg, Some(&in_value.layout)).to_string());
                    } else {
                        constraints.push(format!("{}", op_idx[&idx]));
                    }
                }
                InlineAsmOperandRef::SymFn { instance } => {
                    inputs.push(self.cx().get_fn_addr(instance));
                    op_idx.insert(idx, constraints.len());
                    constraints.push("s".to_string());
                }
                InlineAsmOperandRef::SymStatic { def_id } => {
                    inputs.push(self.cx().get_static(def_id));
                    op_idx.insert(idx, constraints.len());
                    constraints.push("s".to_string());
                }
                _ => {}
            }
        }

        constraints.append(&mut clobbers);

        let volatile = !options.contains(InlineAsmOptions::PURE);
        let alignstack = !options.contains(InlineAsmOptions::NOSTACK);
        let output_type = match &output_types[..] {
            [] => self.type_void(),
            [ty] => *ty,
            tys => self.type_struct(tys, false),
        };

        // END OF COPY

        let (template_string, labels) =
            build_template_string(template, operands, &op_idx, asm_arch, &mut constraints);
        println!("template string: {}\n\n", template_string);

        if labels.len() > 0 {
            panic!("inline assembly with labels is not supported yet: {:?}", labels);
        }

        let result = self.inline_asm_call(template_string, constraints, operands, options);
        let Some(result) = result else {
            panic!("inline_asm_call returned None");
        };

        // BEGIN COPY

        // Switch to the 'normal' basic block if we did an `invoke` instead of a `call`
        if let Some(dest) = dest {
            self.switch_to_block(dest);
        }

        // Write results to outputs
        for (idx, op) in operands.iter().enumerate() {
            if let InlineAsmOperandRef::Out { reg, place: Some(place), .. }
            | InlineAsmOperandRef::InOut { reg, out_place: Some(place), .. } = *op
            {
                let value = if output_types.len() == 1 {
                    result
                } else {
                    self.extract_value(result, op_idx[&idx] as u64)
                };
                let value = llvm_fixup_output(self, value, reg.reg_class(), &place.layout);
                OperandValue::Immediate(value).store(self, place);
            }
        }

        // END COPY

        // panic!("template: {:?}\n\noperands: {}\n\noptions: {:?}\n\nline_spans: {:?}\n\ninstance: {:?}\n\ndest: {:?}\n\ncatch_funclet: {:?}\n\n",
        // template, operands_str, options, line_spans, instance, dest, catch_funclet);
    }
}

fn build_template_string<'m, 'tcx>(
    template: &[rustc_ast::InlineAsmTemplatePiece],
    operands: &[InlineAsmOperandRef<'tcx, Builder<'_, 'm, 'tcx>>],
    op_idx: &FxHashMap<usize, usize>,
    asm_arch: InlineAsmArch,
    constraints: &mut Vec<String>,
) -> (String, Vec<&'m BasicBlock<'m>>) {
    // Build the template string
    let mut labels = vec![];
    let mut template_str = String::new();
    for piece in template {
        match *piece {
            InlineAsmTemplatePiece::String(ref s) => {
                if s.contains('$') {
                    for c in s.chars() {
                        if c == '$' {
                            template_str.push_str("$$");
                        } else {
                            template_str.push(c);
                        }
                    }
                } else {
                    template_str.push_str(s)
                }
            }
            InlineAsmTemplatePiece::Placeholder { operand_idx, modifier, span: _ } => {
                match operands[operand_idx] {
                    InlineAsmOperandRef::In { reg, .. }
                    | InlineAsmOperandRef::Out { reg, .. }
                    | InlineAsmOperandRef::InOut { reg, .. } => {
                        template_str.push_str(&format!("${}", op_idx[&operand_idx]));
                    }
                    InlineAsmOperandRef::Const { ref string } => {
                        // Const operands get injected directly into the template
                        template_str.push_str(string);
                    }
                    InlineAsmOperandRef::SymFn { .. } | InlineAsmOperandRef::SymStatic { .. } => {
                        // Only emit the raw symbol name
                        template_str.push_str(&format!("${}", op_idx[&operand_idx]));
                    }
                    InlineAsmOperandRef::Label { label } => {
                        template_str.push_str(&format!("${}", constraints.len()));
                        constraints.push("!i".to_owned());
                        labels.push(label);
                    }
                }
            }
        }
    }
    (template_str, labels)
    // let mut template_string = String::new();
    // for piece in template {
    //     match piece {
    //         rustc_ast::InlineAsmTemplatePiece::String(s) => {
    //             template_string.push_str(s);
    //         }
    //         rustc_ast::InlineAsmTemplatePiece::Placeholder {
    //             operand_idx,
    //             modifier,
    //             span,
    //         } => {
    //             if let Some(modifier) = modifier {
    //                 template_string.push_str(&format!("${{{}: {}}}", operand_idx, modifier));
    //             } else {
    //                 template_string.push_str(&format!("${{{}}}", operand_idx));
    //             }
    //         }
    //     }
    // }
    // template_string
}

impl<'m, 'tcx> Builder<'_, 'm, 'tcx> {
    pub(crate) fn inline_asm_call(
        &mut self,
        template: String,
        constraints: Vec<String>,
        operands: &[InlineAsmOperandRef<'tcx, Self>],
        options: rustc_ast::InlineAsmOptions,
    ) -> Option<Val<'m>> {
        let nvvm_operands = operands.iter().map(|op| self.operand_to_nvvm(op)).collect::<Vec<_>>();

        let mut output = None;
        for op in &nvvm_operands {
            match op {
                InlineAsmNVVMOperand::Out(ty) => {
                    if output.is_some() {
                        panic!("inline_asm_call: multiple outputs found");
                    }
                    output = Some(*ty);
                }
                _ => {}
            }
        }

        let instr =
            Instruction::InlineAsm { template, constraints, operands: nvvm_operands, output };
        let v = self.cx().get_module_mut().create_val(ValueNVVM::Instr(instr), output);
        self.basic_block.add_instr(v);
        Some(v)
    }

    fn operand_to_nvvm(
        &self,
        operand: &InlineAsmOperandRef<'tcx, Builder<'_, 'm, 'tcx>>,
    ) -> InlineAsmNVVMOperand<'m> {
        match *operand {
            InlineAsmOperandRef::In { reg, value } => InlineAsmNVVMOperand::In(value.immediate()),
            InlineAsmOperandRef::Out { reg, late, place } => {
                let ty = self.cx().lower_layout(place.unwrap().layout);
                InlineAsmNVVMOperand::Out(ty)
            }
            _ => {
                todo!() // Handle other operand types as needed
            }
        }
    }
}

// COPIED AND MODIFIED FROM rustc_codegen_llvm/asm.rs

/// If the register is an xmm/ymm/zmm register then return its index.
fn xmm_reg_index(reg: InlineAsmReg) -> Option<u32> {
    match reg {
        InlineAsmReg::X86(reg)
            if reg as u32 >= X86InlineAsmReg::xmm0 as u32
                && reg as u32 <= X86InlineAsmReg::xmm15 as u32 =>
        {
            Some(reg as u32 - X86InlineAsmReg::xmm0 as u32)
        }
        InlineAsmReg::X86(reg)
            if reg as u32 >= X86InlineAsmReg::ymm0 as u32
                && reg as u32 <= X86InlineAsmReg::ymm15 as u32 =>
        {
            Some(reg as u32 - X86InlineAsmReg::ymm0 as u32)
        }
        InlineAsmReg::X86(reg)
            if reg as u32 >= X86InlineAsmReg::zmm0 as u32
                && reg as u32 <= X86InlineAsmReg::zmm31 as u32 =>
        {
            Some(reg as u32 - X86InlineAsmReg::zmm0 as u32)
        }
        _ => None,
    }
}

/// If the register is an AArch64 integer register then return its index.
fn a64_reg_index(reg: InlineAsmReg) -> Option<u32> {
    match reg {
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x0) => Some(0),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x1) => Some(1),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x2) => Some(2),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x3) => Some(3),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x4) => Some(4),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x5) => Some(5),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x6) => Some(6),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x7) => Some(7),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x8) => Some(8),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x9) => Some(9),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x10) => Some(10),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x11) => Some(11),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x12) => Some(12),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x13) => Some(13),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x14) => Some(14),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x15) => Some(15),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x16) => Some(16),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x17) => Some(17),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x18) => Some(18),
        // x19 is reserved
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x20) => Some(20),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x21) => Some(21),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x22) => Some(22),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x23) => Some(23),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x24) => Some(24),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x25) => Some(25),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x26) => Some(26),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x27) => Some(27),
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x28) => Some(28),
        // x29 is reserved
        InlineAsmReg::AArch64(AArch64InlineAsmReg::x30) => Some(30),
        _ => None,
    }
}

/// If the register is an AArch64 vector register then return its index.
fn a64_vreg_index(reg: InlineAsmReg) -> Option<u32> {
    match reg {
        InlineAsmReg::AArch64(reg)
            if reg as u32 >= AArch64InlineAsmReg::v0 as u32
                && reg as u32 <= AArch64InlineAsmReg::v31 as u32 =>
        {
            Some(reg as u32 - AArch64InlineAsmReg::v0 as u32)
        }
        _ => None,
    }
}

fn is_float(layout: &TyAndLayout<'_>) -> bool {
    match layout.abi {
        Abi::Scalar(Scalar::Initialized { value, .. } | Scalar::Union { value }) => match value {
            Primitive::F32 | Primitive::F64 => true,
            _ => false,
        },
        _ => false,
    }
}

fn normal_register(layout: &TyAndLayout<'_>) -> &'static str {
    if is_float(layout) {
        match layout.size.bytes() {
            4 => "f",
            8 => "d",
            _ => panic!("Unsupported float size: {}", layout.size.bytes()),
        }
    } else {
        match layout.size.bytes() {
            1 => "c",
            2 => "h",
            4 => "r",
            8 => "l",
            _ => panic!("Unsupported integer size: {}", layout.size.bytes()),
        }
    }
}

/// Converts a register class to an LLVM constraint code.
fn reg_to_llvm(reg: InlineAsmRegOrRegClass, layout: Option<&TyAndLayout<'_>>) -> String {
    let is_float = layout.map(|l| is_float(l)).unwrap_or(false);

    match reg {
        // For vector registers LLVM wants the register name to match the type size.
        InlineAsmRegOrRegClass::Reg(reg) => {
            if let Some(idx) = xmm_reg_index(reg) {
                let class = if let Some(layout) = layout {
                    match layout.size.bytes() {
                        64 => 'z',
                        32 => 'y',
                        _ => 'x',
                    }
                } else {
                    // We use f32 as the type for discarded outputs
                    'x'
                };
                format!("{{{}mm{}}}", class, idx)
            } else if let Some(idx) = a64_reg_index(reg) {
                let class = if let Some(layout) = layout {
                    match layout.size.bytes() {
                        8 => 'x',
                        _ => 'w',
                    }
                } else {
                    // We use i32 as the type for discarded outputs
                    'w'
                };
                if class == 'x' && reg == InlineAsmReg::AArch64(AArch64InlineAsmReg::x30) {
                    // LLVM doesn't recognize x30. use lr instead.
                    "{lr}".to_string()
                } else {
                    format!("{{{}{}}}", class, idx)
                }
            } else if let Some(idx) = a64_vreg_index(reg) {
                let class = if let Some(layout) = layout {
                    match layout.size.bytes() {
                        16 => 'q',
                        8 => 'd',
                        4 => 's',
                        2 => 'h',
                        1 => 'd', // We fixup i8 to i8x8
                        _ => unreachable!(),
                    }
                } else {
                    // We use i64x2 as the type for discarded outputs
                    'q'
                };
                format!("{{{}{}}}", class, idx)
            } else if reg == InlineAsmReg::Arm(ArmInlineAsmReg::r14) {
                // LLVM doesn't recognize r14
                "{lr}".to_string()
            } else {
                format!("{{{}}}", reg.name())
            }
        }

        /*
        NVVM-IR constraints:
        c   i8
        h   i16
        r   i32
        l   i64
        f   f32
        d   f64
         */
        InlineAsmRegOrRegClass::RegClass(reg) => {
            match reg {
                InlineAsmRegClass::X86(X86InlineAsmRegClass::reg) => {
                    // depending on the layout, we use different constraints
                    normal_register(layout.expect("Expected layout for normal register"))
                        .to_string()
                }
                _ => panic!("Unsupported register class: {:?}", reg),
            }
        }
    }
}

/// Helper function to get the LLVM type for a Scalar. Pointers are returned as
/// the equivalent integer type.
fn llvm_asm_scalar_type<'m>(cx: &CodegenCx<'m, '_>, scalar: Scalar) -> TyNVVM<'m> {
    let dl = &cx.tcx().data_layout;
    match scalar.primitive() {
        Primitive::Int(Integer::I8, _) => cx.type_i8(),
        Primitive::Int(Integer::I16, _) => cx.type_i16(),
        Primitive::Int(Integer::I32, _) => cx.type_i32(),
        Primitive::Int(Integer::I64, _) => cx.type_i64(),
        Primitive::F32 => cx.type_f32(),
        Primitive::F64 => cx.type_f64(),
        // FIXME(erikdesjardins): handle non-default addrspace ptr sizes
        Primitive::Pointer(_) => cx.type_from_integer(dl.ptr_sized_integer()),
        _ => unreachable!(),
    }
}

/// Fix up an input value to work around LLVM bugs.
fn llvm_fixup_input<'m, 'tcx>(
    bx: &mut Builder<'_, 'm, 'tcx>,
    mut value: Val<'m>,
    reg: InlineAsmRegClass,
    layout: &TyAndLayout<'tcx>,
) -> Val<'m> {
    let dl = &bx.tcx().data_layout;
    match (reg, layout.abi) {
        (InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg), Abi::Scalar(s)) => {
            if let Primitive::Int(Integer::I8, _) = s.primitive() {
                let vec_ty = bx.cx().type_array(bx.cx().type_i8(), 8);
                bx.insert_element(bx.const_undef(vec_ty), value, bx.const_i32(0))
            } else {
                value
            }
        }
        (InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16), Abi::Scalar(s)) => {
            let elem_ty = llvm_asm_scalar_type(bx.cx(), s);
            let count = 16 / layout.size.bytes();
            let vec_ty = bx.cx().type_array(elem_ty, count);
            // FIXME(erikdesjardins): handle non-default addrspace ptr sizes
            if let Primitive::Pointer(_) = s.primitive() {
                let t = bx.type_from_integer(dl.ptr_sized_integer());
                value = bx.ptrtoint(value, t);
            }
            bx.insert_element(bx.const_undef(vec_ty), value, bx.const_i32(0))
        }
        (
            InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16),
            Abi::Vector { element, count },
        ) if layout.size.bytes() == 8 => {
            let elem_ty = llvm_asm_scalar_type(bx.cx(), element);
            let vec_ty = bx.cx().type_array(elem_ty, count);
            let indices: Vec<_> = (0..count * 2).map(|x| bx.const_i32(x as i32)).collect();
            bx.shuffle_vector(value, bx.const_undef(vec_ty), bx.const_array(&indices))
        }
        (InlineAsmRegClass::X86(X86InlineAsmRegClass::reg_abcd), Abi::Scalar(s))
            if s.primitive() == Primitive::F64 =>
        {
            bx.bitcast(value, bx.cx().type_i64())
        }
        (
            InlineAsmRegClass::X86(X86InlineAsmRegClass::xmm_reg | X86InlineAsmRegClass::zmm_reg),
            Abi::Vector { .. },
        ) if layout.size.bytes() == 64 => {
            bx.bitcast(value, bx.cx().type_array(bx.cx().type_f64(), 8))
        }
        (
            InlineAsmRegClass::Arm(ArmInlineAsmRegClass::sreg | ArmInlineAsmRegClass::sreg_low16),
            Abi::Scalar(s),
        ) => {
            if let Primitive::Int(Integer::I32, _) = s.primitive() {
                bx.bitcast(value, bx.cx().type_f32())
            } else {
                value
            }
        }
        (
            InlineAsmRegClass::Arm(
                ArmInlineAsmRegClass::dreg
                | ArmInlineAsmRegClass::dreg_low8
                | ArmInlineAsmRegClass::dreg_low16,
            ),
            Abi::Scalar(s),
        ) => {
            if let Primitive::Int(Integer::I64, _) = s.primitive() {
                bx.bitcast(value, bx.cx().type_f64())
            } else {
                value
            }
        }
        (InlineAsmRegClass::Mips(MipsInlineAsmRegClass::reg), Abi::Scalar(s)) => {
            match s.primitive() {
                // MIPS only supports register-length arithmetics.
                Primitive::Int(Integer::I8 | Integer::I16, _) => bx.zext(value, bx.cx().type_i32()),
                Primitive::F32 => bx.bitcast(value, bx.cx().type_i32()),
                Primitive::F64 => bx.bitcast(value, bx.cx().type_i64()),
                _ => value,
            }
        }
        _ => value,
    }
}

/// Fix up an output value to work around LLVM bugs.
fn llvm_fixup_output<'m, 'tcx>(
    bx: &mut Builder<'_, 'm, 'tcx>,
    mut value: Val<'m>,
    reg: InlineAsmRegClass,
    layout: &TyAndLayout<'tcx>,
) -> Val<'m> {
    match (reg, layout.abi) {
        (InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg), Abi::Scalar(s)) => {
            if let Primitive::Int(Integer::I8, _) = s.primitive() {
                bx.extract_element(value, bx.cx().const_i32(0))
            } else {
                value
            }
        }
        (InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16), Abi::Scalar(s)) => {
            value = bx.extract_element(value, bx.cx().const_i32(0));
            if let Primitive::Pointer(_) = s.primitive() {
                value = bx.inttoptr(value, bx.cx().lower_layout(*layout));
            }
            value
        }
        (
            InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16),
            Abi::Vector { element, count },
        ) if layout.size.bytes() == 8 => {
            let elem_ty = llvm_asm_scalar_type(bx.cx(), element);
            let vec_ty = bx.cx().type_array(elem_ty, count * 2);
            let indices: Vec<_> = (0..count).map(|x| bx.cx().const_i32(x as i32)).collect();
            bx.shuffle_vector(value, bx.cx().const_undef(vec_ty), bx.cx().const_array(&indices))
        }
        (InlineAsmRegClass::X86(X86InlineAsmRegClass::reg_abcd), Abi::Scalar(s))
            if s.primitive() == Primitive::F64 =>
        {
            bx.bitcast(value, bx.cx().type_f64())
        }
        (
            InlineAsmRegClass::X86(X86InlineAsmRegClass::xmm_reg | X86InlineAsmRegClass::zmm_reg),
            Abi::Vector { .. },
        ) if layout.size.bytes() == 64 => bx.bitcast(value, bx.cx().lower_layout(*layout)),
        (
            InlineAsmRegClass::Arm(ArmInlineAsmRegClass::sreg | ArmInlineAsmRegClass::sreg_low16),
            Abi::Scalar(s),
        ) => {
            if let Primitive::Int(Integer::I32, _) = s.primitive() {
                bx.bitcast(value, bx.cx().type_i32())
            } else {
                value
            }
        }
        (
            InlineAsmRegClass::Arm(
                ArmInlineAsmRegClass::dreg
                | ArmInlineAsmRegClass::dreg_low8
                | ArmInlineAsmRegClass::dreg_low16,
            ),
            Abi::Scalar(s),
        ) => {
            if let Primitive::Int(Integer::I64, _) = s.primitive() {
                bx.bitcast(value, bx.cx().type_i64())
            } else {
                value
            }
        }
        (InlineAsmRegClass::Mips(MipsInlineAsmRegClass::reg), Abi::Scalar(s)) => {
            match s.primitive() {
                // MIPS only supports register-length arithmetics.
                Primitive::Int(Integer::I8, _) => bx.trunc(value, bx.cx().type_i8()),
                Primitive::Int(Integer::I16, _) => bx.trunc(value, bx.cx().type_i16()),
                Primitive::F32 => bx.bitcast(value, bx.cx().type_f32()),
                Primitive::F64 => bx.bitcast(value, bx.cx().type_f64()),
                _ => value,
            }
        }
        _ => value,
    }
}

/// Output type to use for llvm_fixup_output.
fn llvm_fixup_output_type<'m, 'tcx>(
    cx: &CodegenCx<'m, 'tcx>,
    reg: InlineAsmRegClass,
    layout: &TyAndLayout<'tcx>,
) -> TyNVVM<'m> {
    match (reg, layout.abi) {
        (InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg), Abi::Scalar(s)) => {
            if let Primitive::Int(Integer::I8, _) = s.primitive() {
                cx.type_array(cx.type_i8(), 8)
            } else {
                //layout.llvm_type(cx)
                cx.lower_layout(*layout)
            }
        }
        (InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16), Abi::Scalar(s)) => {
            let elem_ty = llvm_asm_scalar_type(cx, s);
            let count = 16 / layout.size.bytes();
            cx.type_array(elem_ty, count)
        }
        (
            InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16),
            Abi::Vector { element, count },
        ) if layout.size.bytes() == 8 => {
            let elem_ty = llvm_asm_scalar_type(cx, element);
            cx.type_array(elem_ty, count * 2)
        }
        (InlineAsmRegClass::X86(X86InlineAsmRegClass::reg_abcd), Abi::Scalar(s))
            if s.primitive() == Primitive::F64 =>
        {
            cx.type_i64()
        }
        (
            InlineAsmRegClass::X86(X86InlineAsmRegClass::xmm_reg | X86InlineAsmRegClass::zmm_reg),
            Abi::Vector { .. },
        ) if layout.size.bytes() == 64 => cx.type_array(cx.type_f64(), 8),
        (
            InlineAsmRegClass::Arm(ArmInlineAsmRegClass::sreg | ArmInlineAsmRegClass::sreg_low16),
            Abi::Scalar(s),
        ) => {
            if let Primitive::Int(Integer::I32, _) = s.primitive() {
                cx.type_f32()
            } else {
                //layout.llvm_type(cx)
                cx.lower_layout(*layout)
            }
        }
        (
            InlineAsmRegClass::Arm(
                ArmInlineAsmRegClass::dreg
                | ArmInlineAsmRegClass::dreg_low8
                | ArmInlineAsmRegClass::dreg_low16,
            ),
            Abi::Scalar(s),
        ) => {
            if let Primitive::Int(Integer::I64, _) = s.primitive() {
                cx.type_f64()
            } else {
                //layout.llvm_type(cx)
                cx.lower_layout(*layout)
            }
        }
        (InlineAsmRegClass::Mips(MipsInlineAsmRegClass::reg), Abi::Scalar(s)) => {
            match s.primitive() {
                // MIPS only supports register-length arithmetics.
                Primitive::Int(Integer::I8 | Integer::I16, _) => cx.type_i32(),
                Primitive::F32 => cx.type_i32(),
                Primitive::F64 => cx.type_i64(),
                _ => cx.lower_layout(*layout),
            }
        }
        _ => cx.lower_layout(*layout),
    }
}

/// Converts a modifier into LLVM's equivalent modifier.
fn modifier_to_llvm(
    arch: InlineAsmArch,
    reg: InlineAsmRegClass,
    modifier: Option<char>,
) -> Option<char> {
    // The modifiers can be retrieved from
    // https://llvm.org/docs/LangRef.html#asm-template-argument-modifiers
    match reg {
        InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::reg) => modifier,
        InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg)
        | InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16) => {
            if modifier == Some('v') { None } else { modifier }
        }
        InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::preg) => {
            unreachable!("clobber-only")
        }
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::reg) => None,
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::sreg)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::sreg_low16) => None,
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::dreg)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::dreg_low16)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::dreg_low8) => Some('P'),
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::qreg)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::qreg_low8)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::qreg_low4) => {
            if modifier.is_none() {
                Some('q')
            } else {
                modifier
            }
        }
        InlineAsmRegClass::Hexagon(_) => None,
        InlineAsmRegClass::LoongArch(_) => None,
        InlineAsmRegClass::Mips(_) => None,
        InlineAsmRegClass::Nvptx(_) => None,
        InlineAsmRegClass::PowerPC(_) => None,
        InlineAsmRegClass::RiscV(RiscVInlineAsmRegClass::reg)
        | InlineAsmRegClass::RiscV(RiscVInlineAsmRegClass::freg) => None,
        InlineAsmRegClass::RiscV(RiscVInlineAsmRegClass::vreg) => {
            unreachable!("clobber-only")
        }
        InlineAsmRegClass::X86(X86InlineAsmRegClass::reg)
        | InlineAsmRegClass::X86(X86InlineAsmRegClass::reg_abcd) => match modifier {
            None if arch == InlineAsmArch::X86_64 => Some('q'),
            None => Some('k'),
            Some('l') => Some('b'),
            Some('h') => Some('h'),
            Some('x') => Some('w'),
            Some('e') => Some('k'),
            Some('r') => Some('q'),
            _ => unreachable!(),
        },
        InlineAsmRegClass::X86(X86InlineAsmRegClass::reg_byte) => None,
        InlineAsmRegClass::X86(reg @ X86InlineAsmRegClass::xmm_reg)
        | InlineAsmRegClass::X86(reg @ X86InlineAsmRegClass::ymm_reg)
        | InlineAsmRegClass::X86(reg @ X86InlineAsmRegClass::zmm_reg) => match (reg, modifier) {
            (X86InlineAsmRegClass::xmm_reg, None) => Some('x'),
            (X86InlineAsmRegClass::ymm_reg, None) => Some('t'),
            (X86InlineAsmRegClass::zmm_reg, None) => Some('g'),
            (_, Some('x')) => Some('x'),
            (_, Some('y')) => Some('t'),
            (_, Some('z')) => Some('g'),
            _ => unreachable!(),
        },
        InlineAsmRegClass::X86(X86InlineAsmRegClass::kreg) => None,
        InlineAsmRegClass::X86(
            X86InlineAsmRegClass::x87_reg
            | X86InlineAsmRegClass::mmx_reg
            | X86InlineAsmRegClass::kreg0
            | X86InlineAsmRegClass::tmm_reg,
        ) => {
            unreachable!("clobber-only")
        }
        InlineAsmRegClass::Wasm(WasmInlineAsmRegClass::local) => None,
        InlineAsmRegClass::Bpf(_) => None,
        InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg_pair)
        | InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg_iw)
        | InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg_ptr) => match modifier {
            Some('h') => Some('B'),
            Some('l') => Some('A'),
            _ => None,
        },
        InlineAsmRegClass::Avr(_) => None,
        InlineAsmRegClass::S390x(_) => None,
        InlineAsmRegClass::Msp430(_) => None,
        InlineAsmRegClass::SpirV(SpirVInlineAsmRegClass::reg) => {
            panic!("LLVM backend does not support SPIR-V")
        }
        InlineAsmRegClass::M68k(_) => None,
        InlineAsmRegClass::CSKY(_) => None,
        InlineAsmRegClass::Err => unreachable!(),
    }
}

/// Type to use for outputs that are discarded. It doesn't really matter what
/// the type is, as long as it is valid for the constraint code.
fn dummy_output_type<'m>(cx: &CodegenCx<'m, '_>, reg: InlineAsmRegClass) -> TyNVVM<'m> {
    match reg {
        InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg)
        | InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::vreg_low16) => {
            cx.type_array(cx.type_i64(), 2)
        }
        InlineAsmRegClass::AArch64(AArch64InlineAsmRegClass::preg) => {
            unreachable!("clobber-only")
        }
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::sreg)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::sreg_low16) => cx.type_f32(),
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::dreg)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::dreg_low16)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::dreg_low8) => cx.type_f64(),
        InlineAsmRegClass::Arm(ArmInlineAsmRegClass::qreg)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::qreg_low8)
        | InlineAsmRegClass::Arm(ArmInlineAsmRegClass::qreg_low4) => {
            cx.type_array(cx.type_i64(), 2)
        }
        InlineAsmRegClass::Hexagon(HexagonInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::LoongArch(LoongArchInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::LoongArch(LoongArchInlineAsmRegClass::freg) => cx.type_f32(),
        InlineAsmRegClass::Mips(MipsInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::Mips(MipsInlineAsmRegClass::freg) => cx.type_f32(),
        InlineAsmRegClass::Nvptx(NvptxInlineAsmRegClass::reg16) => cx.type_i16(),
        InlineAsmRegClass::Nvptx(NvptxInlineAsmRegClass::reg32) => cx.type_i32(),
        InlineAsmRegClass::Nvptx(NvptxInlineAsmRegClass::reg64) => cx.type_i64(),
        InlineAsmRegClass::PowerPC(PowerPCInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::PowerPC(PowerPCInlineAsmRegClass::reg_nonzero) => cx.type_i32(),
        InlineAsmRegClass::PowerPC(PowerPCInlineAsmRegClass::freg) => cx.type_f64(),
        InlineAsmRegClass::PowerPC(PowerPCInlineAsmRegClass::cr)
        | InlineAsmRegClass::PowerPC(PowerPCInlineAsmRegClass::xer) => {
            unreachable!("clobber-only")
        }
        InlineAsmRegClass::RiscV(RiscVInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::RiscV(RiscVInlineAsmRegClass::freg) => cx.type_f32(),
        InlineAsmRegClass::RiscV(RiscVInlineAsmRegClass::vreg) => {
            unreachable!("clobber-only")
        }
        InlineAsmRegClass::X86(X86InlineAsmRegClass::reg)
        | InlineAsmRegClass::X86(X86InlineAsmRegClass::reg_abcd) => cx.type_i32(),
        InlineAsmRegClass::X86(X86InlineAsmRegClass::reg_byte) => cx.type_i8(),
        InlineAsmRegClass::X86(X86InlineAsmRegClass::xmm_reg)
        | InlineAsmRegClass::X86(X86InlineAsmRegClass::ymm_reg)
        | InlineAsmRegClass::X86(X86InlineAsmRegClass::zmm_reg) => cx.type_f32(),
        InlineAsmRegClass::X86(X86InlineAsmRegClass::kreg) => cx.type_i16(),
        InlineAsmRegClass::X86(
            X86InlineAsmRegClass::x87_reg
            | X86InlineAsmRegClass::mmx_reg
            | X86InlineAsmRegClass::kreg0
            | X86InlineAsmRegClass::tmm_reg,
        ) => {
            unreachable!("clobber-only")
        }
        InlineAsmRegClass::Wasm(WasmInlineAsmRegClass::local) => cx.type_i32(),
        InlineAsmRegClass::Bpf(BpfInlineAsmRegClass::reg) => cx.type_i64(),
        InlineAsmRegClass::Bpf(BpfInlineAsmRegClass::wreg) => cx.type_i32(),
        InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg) => cx.type_i8(),
        InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg_upper) => cx.type_i8(),
        InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg_pair) => cx.type_i16(),
        InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg_iw) => cx.type_i16(),
        InlineAsmRegClass::Avr(AvrInlineAsmRegClass::reg_ptr) => cx.type_i16(),
        InlineAsmRegClass::S390x(
            S390xInlineAsmRegClass::reg | S390xInlineAsmRegClass::reg_addr,
        ) => cx.type_i32(),
        InlineAsmRegClass::S390x(S390xInlineAsmRegClass::freg) => cx.type_f64(),
        InlineAsmRegClass::Msp430(Msp430InlineAsmRegClass::reg) => cx.type_i16(),
        InlineAsmRegClass::M68k(M68kInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::M68k(M68kInlineAsmRegClass::reg_addr) => cx.type_i32(),
        InlineAsmRegClass::M68k(M68kInlineAsmRegClass::reg_data) => cx.type_i32(),
        InlineAsmRegClass::CSKY(CSKYInlineAsmRegClass::reg) => cx.type_i32(),
        InlineAsmRegClass::CSKY(CSKYInlineAsmRegClass::freg) => cx.type_f32(),
        InlineAsmRegClass::SpirV(SpirVInlineAsmRegClass::reg) => {
            panic!("LLVM backend does not support SPIR-V")
        }
        InlineAsmRegClass::Err => unreachable!(),
    }
}

// END OF COPY
