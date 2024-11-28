use crate::arm64::helper;
use crate::Lifter;
use target_lexicon::{Aarch64Architecture, Architecture};
use thiserror::Error;
use tnj::air::instructions::builder::InstructionBuilder;
use tnj::air::instructions::{Blob, BlockParamData, Inst, Value};
use tnj::arch::get_arch;
use tnj::arch::reg::Reg;
use tnj::types::{BOOL, I16, I32, I64, I8};
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{
    ARMv8, DecodeError, Instruction, Opcode, Operand, ShiftStyle, SizeCode,
};

use super::label_resolver;

/// A lifter for AArch64
pub struct AArch64Lifter;

const INSTRUCTION_SIZE: isize = 4;

enum Flag {
    N,
    Z,
    C,
    V,
}

impl Lifter for AArch64Lifter {
    type E = AArch64LifterError;

    fn lift(&self, code: &[u8], _proofs: &[u8]) -> Result<Blob, Self::E> {
        let arch = get_arch(Architecture::Aarch64(Aarch64Architecture::Aarch64)).unwrap();
        let mut blob = Blob::new(arch);
        let mut builder = blob.insert();

        let decoder = <ARMv8 as Arch>::Decoder::default();
        let mut reader = U8Reader::new(code);
        let label_resolver = label_resolver::LabelResolver::new(code, &mut builder, &decoder)?;

        let mut pc: isize = 0;

        loop {
            match decoder.decode(&mut reader) {
                // TODO: Set insert block if insert block exists. Inefficient
                Ok(inst) => {
                    let block_name = helper::get_block_name(pc);
                    let block = label_resolver.get_block_option_by_name(block_name.as_str());
                    if let Some(block) = block {
                        builder.set_insert_block(*block);
                    }

                    println!("{}", inst);
                    match inst.opcode {
                        Opcode::ADC => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let carry = Self::flag_value(&mut builder, Flag::C);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.add(src1, carry, op_type);
                            let val = builder.add(val, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ADD => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.add(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ADR => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let pc = Self::get_pc(&mut builder);
                            let offset = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.add(pc, offset, I64);
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::AND => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.and(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ASRV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let shift_mask = match op_type {
                                I64 => builder.iconst(63),
                                _ => builder.iconst(31),
                            };
                            let shift_val = builder.and(src2, shift_mask, op_type);
                            let val = builder.ashr(src1, shift_val, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::B => {
                            let offset = helper::get_pc_offset_as_int(inst.operands[0]);
                            let next_address = pc + offset;
                            let block = label_resolver.get_block_by_address(next_address);
                            builder.jump(*block, vec![]);
                        }
                        Opcode::BL => {
                            let instruction_size = builder.iconst(4);
                            let pc_reg = Self::get_pc(&mut builder);
                            let return_address = builder.add(pc_reg, instruction_size, I64);
                            let x30 = Self::get_reg_val_by_name(&mut builder, "x30");

                            let offset = helper::get_pc_offset_as_int(inst.operands[0]);
                            let next_address = pc + offset;
                            let block = label_resolver.get_block_by_address(next_address);
                            builder.write_reg(return_address, x30, I64);
                            builder.jump(*block, vec![]);
                        }
                        Opcode::CBZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let src = Self::get_value(&mut builder, inst.operands[0]);
                            let op_type =
                                helper::get_type_by_sizecode(Self::get_size_code(inst.operands[0]));
                            let zero = builder.iconst(0);
                            let condition =
                                builder.icmp(tnj::types::cmp::CmpTy::Eq, src, zero, op_type);

                            let offset = helper::get_pc_offset_as_int(inst.operands[1]);
                            let jump_address = pc + offset;
                            let block = label_resolver.get_block_by_address(jump_address);

                            builder.jumpif(condition, *block, Vec::new(), next_block, Vec::new());
                        }
                        Opcode::CCMP => {
                            let positive_condition_block = builder.create_block(
                                "ccmp_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "ccmp_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let src1 = Self::get_value(&mut builder, inst.operands[0]);
                            let src2 = Self::get_value(&mut builder, inst.operands[1]);
                            let flag_values = Self::get_value(&mut builder, inst.operands[2]);
                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            let op_type =
                                helper::get_type_by_sizecode(Self::get_size_code(inst.operands[0]));
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            builder.set_insert_block(positive_condition_block);
                            let zero = builder.iconst(0);
                            // set n flag
                            let n_mask = builder.iconst(8);
                            let n = builder.and(n_mask, flag_values, op_type);
                            let n_is_set =
                                builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, n, op_type);
                            Self::write_flag(&mut builder, n_is_set.into(), Flag::N);
                            // set z flag
                            let z_mask = builder.iconst(4);
                            let z = builder.and(z_mask, src2, op_type);
                            let z_is_set =
                                builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, z, op_type);
                            Self::write_flag(&mut builder, z_is_set.into(), Flag::Z);
                            // set c flag
                            let c_mask = builder.iconst(2);
                            let c = builder.and(c_mask, src2, op_type);
                            let c_is_set =
                                builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, c, op_type);
                            Self::write_flag(&mut builder, c_is_set.into(), Flag::C);
                            // set v flag
                            let v_mask = builder.iconst(1);
                            let v = builder.and(v_mask, src2, op_type);
                            let v_is_set =
                                builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, v, op_type);
                            Self::write_flag(&mut builder, v_is_set.into(), Flag::V);
                            builder.jump(next_block, Vec::new());

                            // set flags using immediate
                            builder.set_insert_block(negative_condition_block);
                            builder.icmp(tnj::types::cmp::CmpTy::Ne, src1, src2, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSINC => {
                            let positive_condition_block = builder.create_block(
                                "csinc_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "csinc_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // Condition is false
                            builder.set_insert_block(negative_condition_block);
                            let one = builder.iconst(1);
                            let val = builder.add(src2, one, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            builder.write_reg(src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::EOR => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            builder.xor(src1, src2, op_type);
                            builder.write_reg(src1, dst_reg, op_type);
                        }
                        Opcode::LDP => {
                            let (dst_reg1, sz) = Self::get_dst_reg_by_index(&builder, inst, 0);
                            let (dst_reg2, _) = Self::get_dst_reg_by_index(&builder, inst, 1);
                            let address = Self::get_value(&mut builder, inst.operands[2]);
                            let op_type = helper::get_type_by_sizecode(sz);

                            let val1 = builder.load(address, op_type);
                            builder.write_reg(val1, dst_reg1, op_type);
                            let address_offset = match op_type {
                                I64 => builder.iconst(8),
                                _ => builder.iconst(4),
                            };
                            let address = builder.add(address, address_offset, I64);
                            let val2 = builder.load(address, op_type);
                            builder.write_reg(val2, dst_reg2, op_type);
                        }
                        Opcode::LDR => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::LSLV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let shift_mask = match op_type {
                                I64 => builder.iconst(63),
                                _ => builder.iconst(31),
                            };
                            let shift_val = builder.and(src2, shift_mask, op_type);
                            let val = builder.lshl(src1, shift_val, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::LSRV => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let shift_mask = match op_type {
                                I64 => builder.iconst(63),
                                _ => builder.iconst(31),
                            };
                            let shift_val = builder.and(src2, shift_mask, op_type);
                            let val = builder.lshr(src1, shift_val, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::MADD => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let mul_src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let mul_src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let add_src = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(mul_src1, mul_src2, op_type);
                            let val = builder.add(val, add_src, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::MSUB => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let mul_src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let mul_src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let sub_src = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(mul_src1, mul_src2, op_type);
                            let val = builder.sub(sub_src, val, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::NEG => {
                            let zero = builder.iconst(0);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.sub(zero, src, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ORR => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.or(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ORN => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.not(src2, op_type);
                            let val = builder.or(src1, val, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::RET => {}
                        Opcode::SDIV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.idiv(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::STP => {}
                        Opcode::STR => {
                            let (_, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let value = Self::get_value(&mut builder, inst.operands[0]);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            builder.store(value, address, op_type);
                        }
                        Opcode::SUB => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.sub(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::UDIV => {}
                        op => unimplemented!("{}", op),
                    }
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }

            pc += INSTRUCTION_SIZE;
        }

        Ok(blob)
    }
}

impl AArch64Lifter {
    /// Returns the value of a register as a 64-bit value.
    fn get_value(builder: &mut InstructionBuilder, operand: Operand) -> Value {
        println!("{:?}", operand);
        match operand {
            Operand::Register(sz, reg) => Self::reg_val(builder, sz, reg, SpOrZrReg::Zr),
            Operand::RegisterOrSP(sz, reg) => Self::reg_val(builder, sz, reg, SpOrZrReg::Sp),
            Operand::Immediate(n) => builder.iconst(n),
            Operand::Imm64(n) => builder.iconst(n),
            Operand::Imm16(n) => builder.iconst(n),
            Operand::ImmShift(n, s) => builder.iconst((n as u64) << (s as u64)),
            Operand::ImmShiftMSL(n, s) => {
                let (n, s) = (n as u64, s as u64);
                let val = n << s;
                let mask = (1u64 << s) - 1;
                builder.iconst(val | mask)
            }
            Operand::RegShift(style, s, sz, reg) => {
                // 64 bit value, zero extended
                let reg_val = Self::reg_val(builder, sz, reg, SpOrZrReg::Zr);
                let op_type = helper::get_type_by_sizecode(sz);
                let shift_val = builder.iconst(s as u64);
                match style {
                    ShiftStyle::LSL | ShiftStyle::LSR if s == 0 => reg_val,
                    ShiftStyle::LSL => builder.lshl(reg_val, shift_val, op_type).into(),
                    ShiftStyle::LSR => builder.lshr(reg_val, shift_val, op_type).into(),
                    ShiftStyle::ASR => unimplemented!("ASR"),
                    ShiftStyle::ROR => unimplemented!("ROR"),
                    ShiftStyle::UXTB | ShiftStyle::UXTH | ShiftStyle::UXTW | ShiftStyle::UXTX => {
                        reg_val
                    }
                    ShiftStyle::SXTB => {
                        // TODO: for this we might need some optimization later on.
                        let trunc = builder.trunc_i64(reg_val, I8);
                        builder.sext_i8(trunc, op_type).into()
                    }
                    ShiftStyle::SXTH => {
                        let trunc = builder.trunc_i64(reg_val, I16);
                        builder.sext_i16(trunc, op_type).into()
                    }
                    ShiftStyle::SXTW => {
                        let trunc = builder.trunc_i64(reg_val, I32);
                        builder.sext_i32(trunc, op_type).into()
                    }
                    ShiftStyle::SXTX => reg_val,
                }
            }
            Operand::RegRegOffset(rn, rd, sz, style, s) => {
                let rn = Self::reg_val(builder, sz, rn, SpOrZrReg::Sp);
                let rd = Self::reg_val(builder, sz, rd, SpOrZrReg::Zr);
                let s = builder.iconst(if s == 1 { 2 } else { 0 });
                let op_type = helper::get_type_by_sizecode(sz);
                let offset_val = match style {
                    ShiftStyle::LSL => builder.lshl(rd, s, op_type).into(),
                    ShiftStyle::UXTW => rd,
                    ShiftStyle::SXTW => {
                        let trunc = builder.trunc_i64(rd, I32);
                        builder.sext_i32(trunc, I64).into()
                    }
                    ShiftStyle::SXTX => rd,
                    style => unimplemented!("RegRegOffset with style: {:?}", style),
                };
                builder.add(rn, offset_val, I64).into()
            }
            Operand::RegPreIndex(rn, offset, _write_back) => {
                let rn = Self::reg_val(builder, SizeCode::X, rn, SpOrZrReg::Sp);
                let offset = builder.iconst(offset as u64);
                builder.add(rn, offset, I64).into()
            }
            Operand::RegPostIndex(rn, offset) => {
                let val = Self::reg_val(builder, SizeCode::X, rn, SpOrZrReg::Sp);
                let offset = builder.iconst(offset as u64);
                builder.add(val, offset, I64).into()
            }
            Operand::RegPostIndexReg(_, _) => unimplemented!("RegPostIndexReg"),
            Operand::PCOffset(n) => builder.iconst(n as u64),
            op => unreachable!("incorrect operand for `get_reg_value`: {:?}", op),
        }
    }

    /// reads a register value
    fn reg_val(
        builder: &mut InstructionBuilder,
        sz: SizeCode,
        reg: u16,
        sp_or_zr: SpOrZrReg,
    ) -> Value {
        let op_type = helper::get_type_by_sizecode(sz);
        let val = if reg == 31 {
            match sp_or_zr {
                SpOrZrReg::Sp => builder.read_reg(
                    builder
                        .get_blob()
                        .get_arch()
                        .lookup_reg(&"sp".into())
                        .unwrap(),
                    I64,
                ),
                SpOrZrReg::Zr => {
                    // here we directly return the value as 64 bits
                    return builder.iconst(0);
                }
            }
        } else {
            builder.read_reg(Reg(reg as u32), op_type)
        };
        val.into()
    }

    fn get_dst_reg_by_index(
        builder: &InstructionBuilder,
        inst: Instruction,
        index: usize,
    ) -> (Reg, SizeCode) {
        let (dst_reg, sz) = match inst.operands[index] {
            Operand::Register(sz, reg) => {
                assert_ne!(reg, 31, "cannot write to xzr/wzr");
                (Reg(reg as u32), sz)
            }
            Operand::RegisterOrSP(sz, reg) => {
                if reg == 31 {
                    assert_eq!(sz, SizeCode::X, "sp must be 64 bits");
                    (
                        builder
                            .get_blob()
                            .get_arch()
                            .lookup_reg(&"sp".into())
                            .unwrap(),
                        sz,
                    )
                } else {
                    (Reg(reg as u32), sz)
                }
            }
            op => unimplemented!("dst op {:?}", op),
        };
        (dst_reg, sz)
    }

    fn get_dst_reg(builder: &InstructionBuilder, inst: Instruction) -> (Reg, SizeCode) {
        Self::get_dst_reg_by_index(builder, inst, 0)
    }

    fn flag_value(builder: &mut InstructionBuilder, flag: Flag) -> Value {
        let reg = match flag {
            Flag::N => "n",
            Flag::Z => "z",
            Flag::C => "c",
            Flag::V => "v",
        };
        builder
            .read_reg(
                builder
                    .get_blob()
                    .get_arch()
                    .lookup_reg(&reg.into())
                    .unwrap(),
                BOOL,
            )
            .into()
    }

    fn write_flag(builder: &mut InstructionBuilder, value: Value, flag: Flag) {
        let reg_name = match flag {
            Flag::N => "n",
            Flag::Z => "z",
            Flag::C => "c",
            Flag::V => "v",
        };
        let reg = Self::get_reg_val_by_name(builder, reg_name);
        builder.write_reg(value, reg, BOOL);
    }

    fn get_reg_val_by_name(builder: &mut InstructionBuilder, name: &str) -> Reg {
        builder
            .get_blob()
            .get_arch()
            .lookup_reg(&name.into())
            .unwrap()
    }

    fn get_pc(builder: &mut InstructionBuilder) -> Value {
        let reg = Self::get_reg_val_by_name(builder, "pc");
        Self::reg_val(builder, SizeCode::X, reg.0 as u16, SpOrZrReg::Sp)
    }

    fn get_condition(
        builder: &mut InstructionBuilder,
        operand: Operand,
    ) -> Result<Inst, AArch64LifterError> {
        let flag_is_true = builder.iconst(1);
        let flag_is_false = builder.iconst(0);
        match operand {
            Operand::ConditionCode(cc) => {
                let inst = match cc {
                    0 => {
                        // EQ
                        let z = Self::flag_value(builder, Flag::Z);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, z, flag_is_true, BOOL)
                    }
                    1 => {
                        // NE
                        let z = Self::flag_value(builder, Flag::Z);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, z, flag_is_true, BOOL)
                    }
                    2 => {
                        // CS
                        let c = Self::flag_value(builder, Flag::C);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, c, flag_is_true, BOOL)
                    }
                    3 => {
                        // CC
                        let c = Self::flag_value(builder, Flag::C);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, c, flag_is_true, BOOL)
                    }
                    4 => {
                        // MI
                        let n = Self::flag_value(builder, Flag::N);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, n, flag_is_true, BOOL)
                    }
                    5 => {
                        // PL
                        let n = Self::flag_value(builder, Flag::N);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, n, flag_is_true, BOOL)
                    }
                    6 => {
                        // VS
                        let v = Self::flag_value(builder, Flag::V);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, v, flag_is_true, BOOL)
                    }
                    7 => {
                        // VC
                        let v = Self::flag_value(builder, Flag::V);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, v, flag_is_true, BOOL)
                    }
                    8 => {
                        // HI
                        let z = Self::flag_value(builder, Flag::Z);
                        let c = Self::flag_value(builder, Flag::C);

                        let c_is_true =
                            builder.icmp(tnj::types::cmp::CmpTy::Eq, c, flag_is_true, BOOL);
                        let z_is_false =
                            builder.icmp(tnj::types::cmp::CmpTy::Ne, z, flag_is_true, BOOL);
                        builder.and(c_is_true, z_is_false, BOOL)
                    }
                    9 => {
                        // LS
                        let z = Self::flag_value(builder, Flag::Z);
                        let c = Self::flag_value(builder, Flag::C);

                        let c_is_false: Inst =
                            builder.icmp(tnj::types::cmp::CmpTy::Ne, c, flag_is_true, BOOL);

                        let z_is_true =
                            builder.icmp(tnj::types::cmp::CmpTy::Eq, z, flag_is_true, BOOL);
                        builder.or(c_is_false, z_is_true, BOOL)
                    }
                    10 => {
                        // GE
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        let n_eq_v = builder.icmp(tnj::types::cmp::CmpTy::Eq, n, v, BOOL);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, n_eq_v, flag_is_true, BOOL)
                    }
                    11 => {
                        // LT
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        let n_neq_v = builder.icmp(tnj::types::cmp::CmpTy::Ne, n, v, BOOL);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, n_neq_v, flag_is_true, BOOL)
                    }
                    12 => {
                        // GT
                        let z = Self::flag_value(builder, Flag::Z);
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        let z_is_false =
                            builder.icmp(tnj::types::cmp::CmpTy::Ne, z, flag_is_true, BOOL);
                        let n_eq_v = builder.icmp(tnj::types::cmp::CmpTy::Eq, n, v, BOOL);
                        builder.and(z_is_false, n_eq_v, BOOL)
                    }
                    13 => {
                        // LE
                        let z = Self::flag_value(builder, Flag::Z);
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        let z_is_true =
                            builder.icmp(tnj::types::cmp::CmpTy::Eq, z, flag_is_true, BOOL);
                        let n_neq_v = builder.icmp(tnj::types::cmp::CmpTy::Ne, n, v, BOOL);
                        builder.or(z_is_true, n_neq_v, BOOL)
                    }
                    14 => {
                        // AL
                        builder.and(flag_is_true, flag_is_true, BOOL)
                    }
                    15 => {
                        // NV
                        builder.and(flag_is_true, flag_is_false, BOOL)
                    }
                    _ => {
                        return Err(AArch64LifterError::CustomError(
                            "Invalid operand for condition code".to_string(),
                        ))
                    }
                };
                Ok(inst)
            }
            _ => Err(AArch64LifterError::CustomError(
                "Invalid operand for condition code".to_string(),
            )),
        }
    }

    fn get_size_code(operand: Operand) -> SizeCode {
        match operand {
            Operand::Register(sz, _) => sz,
            Operand::RegisterOrSP(sz, _) => sz,
            Operand::RegShift(_, _, sz, _) => sz,
            Operand::RegRegOffset(_, _, sz, _, _) => sz,
            _ => panic!("can not get size code for non-register types"),
        }
    }
}

/// Whether reg 31 refers to register sp or reg zero
enum SpOrZrReg {
    Sp,
    Zr,
}

/// Error type for lifting from machine code to AIR
#[derive(Debug, Error)]
pub enum AArch64LifterError {
    /// Error decoding the instructions
    #[error("Error decoding machine code: {0}")]
    DecodeError(#[from] DecodeError),

    /// Custom error with message
    #[error("{0}")]
    CustomError(String),
}
