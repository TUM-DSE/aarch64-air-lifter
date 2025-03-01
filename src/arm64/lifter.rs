use crate::arm64::helper;
use crate::Lifter;
use target_lexicon::{Aarch64Architecture, Architecture};
use thiserror::Error;
use tnj::air::instructions::builder::InstructionBuilder;
use tnj::air::instructions::{Blob, Inst, Value};
use tnj::arch::get_arch;
use tnj::arch::reg::Reg;
use tnj::types::{Type, BOOL, I1, I128, I16, I32, I64, I8};
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{
    ARMv8, DecodeError, Instruction, Opcode, Operand, ShiftStyle, SizeCode,
};

use super::label_resolver;

/// A lifter for AArch64
pub struct AArch64Lifter;

const INSTRUCTION_SIZE: u64 = 4;

enum Flag {
    N,
    Z,
    C,
    V,
}

impl AArch64Lifter {
    /// Disassemble code and print to a string.
    pub fn disassemble<W>(&self, w: &mut W, code: &[u8]) -> Result<(), AArch64DisassemblerError>
    where
        W: ?Sized + std::io::Write,
    {
        let decoder = <ARMv8 as Arch>::Decoder::default();
        let mut reader = U8Reader::new(code);

        let mut pc = 0u64;

        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    writeln!(w, "0x{:0>4x}:\t{}", pc, inst)?;
                    pc += INSTRUCTION_SIZE;
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64DisassemblerError::DecodeError(e)),
            }
        }

        Ok(())
    }
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

        let mut pc: u64 = 0;

        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    let block_name = helper::get_block_name(pc);
                    let block = label_resolver.get_block_option_by_name(block_name.as_str());
                    if let Some(block) = block {
                        builder.jump(*block, vec![]);
                        builder.set_insert_block(*block);
                    }

                    match inst.opcode {
                        Opcode::ADC | Opcode::ADCS => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let carry = Self::flag_value(&mut builder, Flag::C);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.add(src1, carry, op_type);
                            let val = builder.add(val, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);

                            if inst.opcode == Opcode::ADCS {
                                Self::set_flags_using_adc(&mut builder, src1, src2, op_type, carry);
                            }
                        }
                        Opcode::ADD | Opcode::ADDS => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.add(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);

                            if inst.opcode == Opcode::ADDS {
                                let zero = builder.iconst(0);
                                Self::set_flags_using_adc(&mut builder, src1, src2, op_type, zero);
                            }
                        }
                        Opcode::ADR => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let pc = Self::get_pc(&mut builder);
                            let offset = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.add(pc, offset, I64);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        }
                        Opcode::ADRP => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let offset = Self::get_value(&mut builder, inst.operands[1]);
                            let reverse_mask = builder.iconst(0xFFF);
                            let mask = builder.bitwise_not(reverse_mask, I64);
                            let pc = Self::get_pc(&mut builder);
                            let masked_pc = builder.and(pc, mask, I64);
                            let addr = builder.add(masked_pc, offset, I64);
                            Self::write_reg(&mut builder, addr, dst_reg, I64);
                        }
                        Opcode::AND | Opcode::ANDS => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.and(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);

                            if inst.opcode == Opcode::ANDS {
                                let zero = builder.iconst(0);
                                Self::write_flag(&mut builder, zero, Flag::C);
                                Self::write_flag(&mut builder, zero, Flag::V);
                                let is_zero =
                                    builder.icmp(tnj::types::cmp::CmpTy::Eq, val, zero, op_type);
                                Self::write_flag(&mut builder, is_zero.into(), Flag::Z);
                                let is_negative =
                                    builder.icmp(tnj::types::cmp::CmpTy::Slt, val, zero, op_type);
                                Self::write_flag(&mut builder, is_negative.into(), Flag::N);
                            }
                        }
                        Opcode::ASRV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let shift_mask = match op_type {
                                I64 => builder.iconst(63),
                                _ => builder.iconst(31),
                            };
                            let shift_val = builder.and(src2, shift_mask, op_type);
                            let val = builder.ashr(src1, shift_val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::B | Opcode::BL => {
                            if inst.opcode == Opcode::BL {
                                let instruction_size = builder.iconst(4);
                                let pc_reg = Self::get_pc(&mut builder);
                                let return_address = builder.add(pc_reg, instruction_size, I64);
                                let x30 = Self::get_reg_val_by_name(&mut builder, "x30");
                                Self::write_reg(&mut builder, return_address, x30, I64);
                            }
                            let offset = helper::get_pc_offset_as_int(inst.operands[0]);
                            let next_address = (pc as i64).wrapping_add(offset) as u64;
                            let block = label_resolver.get_block_by_address(next_address);
                            builder.jump(*block, vec![]);
                        }
                        Opcode::Bcc(condition) => {
                            let offset = helper::get_pc_offset_as_int(inst.operands[0]);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let jump_block = *label_resolver.get_block_by_address(jump_address);
                            let next_address: u64 = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let operand = Operand::ConditionCode(condition);
                            let condition = Self::get_condition(&mut builder, operand)?;
                            builder.jumpif(
                                condition,
                                jump_block,
                                Vec::new(),
                                next_block,
                                Vec::new(),
                            );
                        }
                        Opcode::BFM => {
                            let positive_condition_block =
                                builder.create_block("bfm_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("bfm_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let immr = Self::get_value(&mut builder, inst.operands[2]);
                            let imms = Self::get_value(&mut builder, inst.operands[3]);
                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Uge, imms, immr, I64);
                            builder.jumpif(
                                cmp,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // copies a bitfield of (<imms>-<immr>+1) bits starting from bit position <immr> in the source register to the least significant bits of the destination register
                            builder.set_insert_block(positive_condition_block);
                            // get src bitfield
                            let one = builder.iconst(1);
                            let src_bitfield_size = builder.add(one, imms, op_type);
                            let src_bitfield_size = builder.sub(src_bitfield_size, immr, op_type);
                            let src_mask = builder.lshl(one, src_bitfield_size, op_type);
                            let src_mask = builder.sub(src_mask, one, op_type);
                            let src_mask = builder.lshl(src_mask, immr, op_type);
                            let src_bitfield = builder.and(src, src_mask, op_type);
                            let src_bitfield = builder.lshr(src_bitfield, immr, op_type);
                            // clear dst bits that are replaced by the src bitfield
                            let dst_mask = builder.lshl(one, src_bitfield_size, op_type);
                            let dst_mask = builder.sub(dst_mask, one, op_type);
                            let dst_mask = builder.bitwise_not(dst_mask, op_type);
                            let dst_bitfield = builder.and(src, dst_mask, op_type);
                            // merge and write bitfield
                            let val = builder.or(src_bitfield, dst_bitfield, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            builder.set_insert_block(negative_condition_block);
                            // get bitfield containing src bits
                            let src_bitfield_size = builder.add(one, imms, op_type);
                            let src_mask = builder.lshl(one, src_bitfield_size, op_type);
                            let src_mask = builder.sub(src_mask, one, op_type);
                            let src_bitfield = builder.and(src, src_mask, op_type);
                            let reg_size = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };
                            let starting_position = builder.sub(reg_size, immr, op_type);
                            let src_bitfield =
                                builder.lshl(src_bitfield, starting_position, op_type);
                            // clear dst bits that are replaced by the src bitfield
                            let dst_mask = builder.lshl(one, src_bitfield_size, op_type);
                            let dst_mask = builder.sub(dst_mask, one, op_type);
                            let dst_mask = builder.lshl(dst_mask, starting_position, op_type);
                            let dst_mask = builder.bitwise_not(dst_mask, op_type);
                            let dst_bitfield = builder.and(src, dst_mask, op_type);
                            // merge and write bitfield
                            let val = builder.or(src_bitfield, dst_bitfield, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::BIC => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let neg_src2 = builder.bitwise_not(src2, op_type);
                            let val = builder.and(src1, neg_src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::BLR | Opcode::BR => {
                            if inst.opcode == Opcode::BLR {
                                let pc = Self::get_pc(&mut builder);
                                let four = builder.iconst(4);
                                let ret_address = builder.add(pc, four, I64);
                                let x30 = Self::get_reg_val_by_name(&mut builder, "x30");
                                Self::write_reg(&mut builder, ret_address, x30, I64);
                            }
                            let address = Self::get_value(&mut builder, inst.operands[0]);
                            builder.dynamic_jump(address);
                            if inst.opcode == Opcode::BLR {
                                builder.invalidate_regs();
                            }
                        }
                        Opcode::CAS(_memory_ordering) => {
                            // Untested
                            let swap_block = builder.create_block("cas_swap", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let old = Self::get_value(&mut builder, inst.operands[0]);
                            let new = Self::get_value(&mut builder, inst.operands[1]);
                            let addr = Self::get_value(&mut builder, inst.operands[2]);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.load(addr, op_type);
                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Eq, val, old, op_type);
                            builder.jumpif(cmp, swap_block, Vec::new(), next_block, Vec::new());

                            builder.set_insert_block(swap_block);
                            builder.store(new, addr, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CBNZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let src = Self::get_value(&mut builder, inst.operands[0]);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = builder.iconst(0);
                            let condition =
                                builder.icmp(tnj::types::cmp::CmpTy::Ne, src, zero, op_type);

                            let offset = helper::get_pc_offset_as_int(inst.operands[1]);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let block = label_resolver.get_block_by_address(jump_address);

                            builder.jumpif(condition, *block, Vec::new(), next_block, Vec::new());
                        }
                        Opcode::CBZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let src = Self::get_value(&mut builder, inst.operands[0]);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = builder.iconst(0);
                            let condition =
                                builder.icmp(tnj::types::cmp::CmpTy::Eq, src, zero, op_type);

                            let offset = helper::get_pc_offset_as_int(inst.operands[1]);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let block = label_resolver.get_block_by_address(jump_address);

                            builder.jumpif(condition, *block, Vec::new(), next_block, Vec::new());
                        }
                        Opcode::CCMN => {
                            let positive_condition_block =
                                builder.create_block("ccmp_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("ccmp_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            let op_type = helper::get_type_by_inst(inst);
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[0]);
                            let src2 = Self::get_value(&mut builder, inst.operands[1]);
                            let carry = builder.iconst(0);
                            Self::set_flags_using_adc(&mut builder, src1, src2, op_type, carry);
                            builder.jump(next_block, Vec::new());

                            builder.set_insert_block(negative_condition_block);
                            let flag_val = Self::get_value(&mut builder, inst.operands[2]);
                            Self::set_flags_to_value(&mut builder, flag_val, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CCMP => {
                            let positive_condition_block =
                                builder.create_block("ccmp_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("ccmp_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            let op_type = helper::get_type_by_inst(inst);
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[0]);
                            let src2 = Self::get_value(&mut builder, inst.operands[1]);
                            let not_src2 = builder.bitwise_not(src2, op_type);
                            let carry = builder.iconst(0);
                            Self::set_flags_using_adc(
                                &mut builder,
                                src1,
                                not_src2.into(),
                                op_type,
                                carry,
                            );
                            builder.jump(next_block, Vec::new());

                            builder.set_insert_block(negative_condition_block);
                            let flag_val = Self::get_value(&mut builder, inst.operands[2]);
                            Self::set_flags_to_value(&mut builder, flag_val, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CLS => {
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);

                            let one = builder.iconst(1);
                            let val1 = builder.lshr(src, one, op_type);
                            let val2_mask = builder.ror(one, one, op_type);
                            let val2_mask = builder.bitwise_not(val2_mask, op_type);
                            let val2 = builder.and(val2_mask, src, op_type);
                            let val = builder.xor(val1, val2, op_type);

                            let n = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };
                            let highest_set_bit = builder.highest_set_bit(val, op_type);
                            let val = builder.sub(n, highest_set_bit, op_type);
                            let val = builder.sub(val, one, op_type);

                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::CLZ => {
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let one = builder.iconst(1);

                            let n = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };
                            let highest_set_bit = builder.highest_set_bit(src, op_type);
                            let val = builder.sub(n, highest_set_bit, op_type);
                            let val = builder.sub(val, one, op_type);

                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::CSEL => {
                            let positive_condition_block =
                                builder.create_block("csel_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("csel_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            Self::write_reg(&mut builder, src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            builder.set_insert_block(negative_condition_block);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            Self::write_reg(&mut builder, src2, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSINC => {
                            let positive_condition_block =
                                builder.create_block("csinc_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("csinc_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            Self::write_reg(&mut builder, src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // Condition is false
                            builder.set_insert_block(negative_condition_block);
                            let one = builder.iconst(1);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let val = builder.add(src2, one, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSINV => {
                            let positive_condition_block =
                                builder.create_block("csinv_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("csinv_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            Self::write_reg(&mut builder, src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // Condition is false
                            builder.set_insert_block(negative_condition_block);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let val = builder.bitwise_not(src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSNEG => {
                            let positive_condition_block =
                                builder.create_block("csneg_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("csneg_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = Self::get_condition(&mut builder, inst.operands[3])?;
                            builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            Self::write_reg(&mut builder, src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // Condition is false
                            builder.set_insert_block(negative_condition_block);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let zero = builder.iconst(0);
                            let val = builder.sub(zero, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::EON => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);

                            let src2 = builder.bitwise_not(src2, op_type);
                            let val = builder.xor(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::EOR => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            builder.xor(src1, src2, op_type);
                            Self::write_reg(&mut builder, src1, dst_reg, op_type);
                        }
                        Opcode::EXTR => {
                            // 4 Operands
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let shift_val = Self::get_value(&mut builder, inst.operands[3]);

                            let datasize = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };
                            let src2 = builder.lshr(src2, shift_val, op_type);
                            let shift_val = builder.sub(datasize, shift_val, op_type);
                            let src1 = builder.lshl(src1, shift_val, op_type);
                            let val = builder.or(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::HINT => {
                            // HINT is a no-op
                        }
                        Opcode::HVC => {
                            // We are ignoring hypervisor calls
                            builder.invalidate_regs();
                        }
                        Opcode::LDP | Opcode::LDXP => {
                            let dst_reg1 = Self::get_reg_by_index(&builder, inst, 0);
                            let dst_reg2 = Self::get_reg_by_index(&builder, inst, 1);
                            let address = Self::get_value(&mut builder, inst.operands[2]);
                            let op_type = helper::get_type_by_inst(inst);

                            let val1 = builder.load(address, op_type);
                            Self::write_reg(&mut builder, val1, dst_reg1, op_type);
                            let address_offset = match op_type {
                                I64 => builder.iconst(8),
                                _ => builder.iconst(4),
                            };
                            let address = builder.add(address, address_offset, I64);
                            let val2 = builder.load(address, op_type);
                            Self::write_reg(&mut builder, val2, dst_reg2, op_type);
                        }
                        Opcode::LDPSW => {
                            let dst_reg1 = Self::get_reg_by_index(&builder, inst, 0);
                            let dst_reg2 = Self::get_reg_by_index(&builder, inst, 1);
                            let address = Self::get_value(&mut builder, inst.operands[2]);

                            let val1 = builder.load(address, I32);
                            let val1 = builder.sext_i32(val1, I64);
                            Self::write_reg(&mut builder, val1, dst_reg1, I64);
                            let address_offset = builder.iconst(4);
                            let address = builder.add(address, address_offset, I64);
                            let val2 = builder.load(address, I32);
                            let val2 = builder.sext_i32(val2, I64);
                            Self::write_reg(&mut builder, val2, dst_reg2, I64);
                        }
                        Opcode::LDR
                        | Opcode::LDUR
                        | Opcode::LDAR
                        | Opcode::LDXR
                        | Opcode::LDAXR
                        | Opcode::LDTR => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::LDRB
                        | Opcode::LDURB
                        | Opcode::LDARB
                        | Opcode::LDXRB
                        | Opcode::LDAXRB
                        | Opcode::LDTRB => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, I8);
                            let val = builder.zext_i8(val, I32);
                            Self::write_reg(&mut builder, val, dst_reg, I32);
                        }
                        Opcode::LDRH
                        | Opcode::LDURH
                        | Opcode::LDARH
                        | Opcode::LDXRH
                        | Opcode::LDAXRH
                        | Opcode::LDTRH => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, I16);
                            let val = builder.zext_i16(val, I32);
                            Self::write_reg(&mut builder, val, dst_reg, I32);
                        }
                        Opcode::LDRSB | Opcode::LDTRSB | Opcode::LDURSB => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, I8);
                            let val = builder.sext_i8(val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::LDRSH | Opcode::LDTRSH | Opcode::LDURSH => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, I16);
                            let val = builder.sext_i16(val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::LDRSW | Opcode::LDTRSW | Opcode::LDURSW => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, I32);
                            let val = builder.sext_i32(val, I64);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        }
                        Opcode::LSLV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let shift_mask = match op_type {
                                I64 => builder.iconst(63),
                                _ => builder.iconst(31),
                            };
                            let shift_val = builder.and(src2, shift_mask, op_type);
                            let val = builder.lshl(src1, shift_val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::LSRV => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let shift_mask = match op_type {
                                I64 => builder.iconst(63),
                                _ => builder.iconst(31),
                            };
                            let shift_val = builder.and(src2, shift_mask, op_type);
                            let val = builder.lshr(src1, shift_val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::MADD => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let mul_src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let mul_src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let add_src = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(mul_src1, mul_src2, op_type);
                            let val = builder.add(val, add_src, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::MOVK => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            Self::write_reg(&mut builder, src, dst_reg, I16);
                        }
                        Opcode::MOVN => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = builder.iconst(0);
                            Self::write_reg(&mut builder, zero, dst_reg, op_type);

                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let src = builder.bitwise_not(src, I16);
                            Self::write_reg(&mut builder, src, dst_reg, I16);
                        }
                        Opcode::MOVZ => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = builder.iconst(0);
                            Self::write_reg(&mut builder, zero, dst_reg, op_type);

                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            Self::write_reg(&mut builder, src, dst_reg, I16);
                        }
                        Opcode::MSUB => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let mul_src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let mul_src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let sub_src = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(mul_src1, mul_src2, op_type);
                            let val = builder.sub(sub_src, val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::NEG => {
                            let zero = builder.iconst(0);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.sub(zero, src, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::ORN => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let val = builder.bitwise_not(src2, op_type);
                            let val = builder.or(src1, val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::ORR => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.or(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::PRFM | Opcode::PRFUM => {
                            // We are ignoring prefetch hints
                        }
                        Opcode::RBIT => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.reverse_bits(src, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::RET | Opcode::RETAB | Opcode::RETAA => {
                            builder.ret();
                        }
                        Opcode::REV | Opcode::REV64 => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.reverse_bytes(src, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::REV16 => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let mut src = Self::get_value(&mut builder, inst.operands[1]);
                            let mut res = builder.iconst(0);
                            let sixteen = builder.iconst(16);

                            let loop_iterations = match op_type {
                                I128 => 8,
                                I64 => 4,
                                _ => 2,
                            };
                            for _ in 0..loop_iterations {
                                let val = builder.reverse_bytes(src, I16);
                                res = builder.or(res, val, I16).into();
                                res = builder.ror(res, sixteen, op_type).into();
                                src = builder.ror(src, sixteen, op_type).into();
                            }
                            Self::write_reg(&mut builder, res, dst_reg, op_type);
                        }
                        Opcode::REV32 => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let mut src = Self::get_value(&mut builder, inst.operands[1]);
                            let mut res = builder.iconst(0);
                            let thirtytwo = builder.iconst(32);

                            let val = builder.reverse_bytes(src, I32);
                            res = builder.or(res, val, I32).into();
                            res = builder.ror(res, thirtytwo, I64).into();
                            src = builder.ror(src, thirtytwo, I64).into();

                            let val = builder.reverse_bytes(src, I32);
                            res = builder.or(res, val, I32).into();
                            res = builder.ror(res, thirtytwo, I64).into();

                            Self::write_reg(&mut builder, res, dst_reg, I64);
                        }
                        Opcode::RORV => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let mask = match op_type {
                                I64 => builder.iconst(63),
                                _ => builder.iconst(31),
                            };
                            let shift = builder.and(src2, mask, op_type);
                            let val = builder.ror(src1, shift, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::SBC | Opcode::SBCS => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let carry = Self::flag_value(&mut builder, Flag::C);
                            let carry = builder.bitwise_not(carry, I1);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.sub(src1, src2, op_type);
                            let val = builder.sub(val, carry, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            if inst.opcode == Opcode::SBCS {
                                let carry = Self::flag_value(&mut builder, Flag::C);
                                Self::set_flags_using_adc(&mut builder, src1, src2, op_type, carry);
                            }
                        }
                        Opcode::SBFM => {
                            let positive_condition_block =
                                builder.create_block("sbfm_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("sbfm_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let immr = Self::get_value(&mut builder, inst.operands[2]);
                            let imms = Self::get_value(&mut builder, inst.operands[3]);
                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Uge, imms, immr, I64);
                            builder.jumpif(
                                cmp,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            let reg_size = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };

                            // copies a bitfield of (<imms>-<immr>+1) bits starting from bit position <immr> in the source register to the least significant bits of the destination register
                            builder.set_insert_block(positive_condition_block);
                            // get src bitfield
                            let one = builder.iconst(1);
                            let src_bitfield_size = builder.add(one, imms, op_type);
                            let src_bitfield_size = builder.sub(src_bitfield_size, immr, op_type);
                            let shift_val = builder.add(imms, one, op_type);
                            let shift_val = builder.sub(reg_size, shift_val, op_type);
                            let val = builder.lshl(src, shift_val, op_type);
                            let shift_val = builder.sub(reg_size, src_bitfield_size, op_type);
                            let val = builder.ashr(val, shift_val, op_type);

                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            builder.set_insert_block(negative_condition_block);
                            let shift_val = builder.add(imms, one, op_type);
                            let shift_val = builder.sub(reg_size, shift_val, op_type);
                            let val = builder.lshl(src, shift_val, op_type);
                            let shift_val = builder.sub(reg_size, immr, op_type);
                            let val = builder.ashr(val, shift_val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::SDIV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = builder.iconst(0);
                            let trap =
                                builder.icmp(tnj::types::cmp::CmpTy::Eq, src2, zero, op_type);
                            builder.trapif(trap);
                            let val = builder.idiv(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::SMADDL => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let src3 = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(src1, src2, I32);
                            let val = builder.add(val, src3, I64);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        }
                        Opcode::SMC => {
                            // Ignoring secure monitor calls
                        }
                        Opcode::SMSUBL => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let src3 = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(src1, src2, I32);
                            let val = builder.sub(src3, val, I64);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        }
                        Opcode::SMULH => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let val = builder.imul(src1, src2, I64);
                            let sixtyfour = builder.iconst(64);
                            let val = builder.ashr(val, sixtyfour, I128);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        }
                        Opcode::STP | Opcode::STNP => {
                            let src1 = Self::get_value(&mut builder, inst.operands[0]);
                            let src2 = Self::get_value(&mut builder, inst.operands[1]);
                            let address = Self::get_value(&mut builder, inst.operands[2]);
                            let op_type = helper::get_type_by_inst(inst);

                            builder.store(src1, address, op_type);
                            let address_offset = match op_type {
                                I64 => builder.iconst(8),
                                _ => builder.iconst(4),
                            };
                            let address = builder.add(address, address_offset, I64);
                            builder.store(src2, address, op_type);
                        }
                        Opcode::STXP | Opcode::STLXP => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let address = Self::get_value(&mut builder, inst.operands[3]);
                            let op_type = helper::get_type_by_inst(inst);

                            builder.store(src1, address, op_type);
                            let address_offset = match op_type {
                                I64 => builder.iconst(8),
                                _ => builder.iconst(4),
                            };
                            let address = builder.add(address, address_offset, I64);
                            builder.store(src2, address, op_type);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let opaque = builder.opaque(op_type);
                            builder.write_reg(opaque, dst_reg, op_type);
                        }
                        Opcode::STR
                        | Opcode::STLR
                        | Opcode::STUR
                        | Opcode::STLUR
                        | Opcode::STTR => {
                            let op_type = helper::get_type_by_inst(inst);
                            let value = Self::get_value(&mut builder, inst.operands[0]);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            builder.store(value, address, op_type);
                        }
                        Opcode::STLXR | Opcode::STXR => {
                            let op_type = helper::get_type_by_inst(inst);
                            let value = Self::get_value(&mut builder, inst.operands[1]);
                            let address = Self::get_value(&mut builder, inst.operands[2]);
                            builder.store(value, address, op_type);
                            let opaque = builder.opaque(op_type);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            Self::write_reg(&mut builder, opaque, dst_reg, op_type);
                        }
                        Opcode::STRB
                        | Opcode::STLRB
                        | Opcode::STURB
                        | Opcode::STLURB
                        | Opcode::STTRB => {
                            let value = Self::get_value(&mut builder, inst.operands[0]);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            builder.store(value, address, I8);
                        }
                        Opcode::STLXRB | Opcode::STXRB => {
                            let value = Self::get_value(&mut builder, inst.operands[1]);
                            let address = Self::get_value(&mut builder, inst.operands[2]);
                            builder.store(value, address, I8);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let opaque = builder.opaque(I8);
                            Self::write_reg(&mut builder, opaque, dst_reg, I8);
                        }
                        Opcode::STRH
                        | Opcode::STLRH
                        | Opcode::STURH
                        | Opcode::STLURH
                        | Opcode::STTRH => {
                            let value = Self::get_value(&mut builder, inst.operands[0]);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            builder.store(value, address, I32);
                        }
                        Opcode::STLXRH | Opcode::STXRH => {
                            let value = Self::get_value(&mut builder, inst.operands[1]);
                            let address = Self::get_value(&mut builder, inst.operands[2]);
                            builder.store(value, address, I32);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let opaque = builder.opaque(I32);
                            Self::write_reg(&mut builder, opaque, dst_reg, I32);
                        }
                        Opcode::SUB | Opcode::SUBS => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = builder.sub(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            if inst.opcode == Opcode::SUBS {
                                let one = builder.iconst(1);
                                let not_src2 = builder.bitwise_not(src2, op_type).into();
                                Self::set_flags_using_adc(
                                    &mut builder,
                                    src1,
                                    not_src2,
                                    op_type,
                                    one,
                                );
                            }
                        }
                        Opcode::SVC => {
                            // Ignoring supervisor calls
                            builder.invalidate_regs();
                        }
                        Opcode::SYS(_data) | Opcode::SYSL(_data) => {
                            // Ignoring system calls
                            builder.invalidate_regs();
                        }
                        Opcode::TBNZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let one = builder.iconst(1);
                            let zero = builder.iconst(0);
                            let src = Self::get_reg_by_index(&builder, inst, 0);
                            let op_type = helper::get_type_by_inst(inst);
                            let test_bit = Self::get_value(&mut builder, inst.operands[1]);
                            let offset = helper::get_pc_offset_as_int(inst.operands[2]);

                            let test_bit = builder.lshr(test_bit, one, op_type);
                            let val = builder.and(test_bit, src, op_type);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let jump_block = *label_resolver.get_block_by_address(jump_address);

                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Ne, val, zero, op_type);
                            builder.jumpif(cmp, jump_block, Vec::new(), next_block, Vec::new());
                        }
                        Opcode::TBZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let one = builder.iconst(1);
                            let zero = builder.iconst(0);
                            let src = Self::get_reg_by_index(&builder, inst, 0);
                            let op_type = helper::get_type_by_inst(inst);
                            let test_bit = Self::get_value(&mut builder, inst.operands[1]);
                            let offset = helper::get_pc_offset_as_int(inst.operands[2]);

                            let test_bit = builder.lshr(test_bit, one, op_type);
                            let val = builder.and(test_bit, src, op_type);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let jump_block = *label_resolver.get_block_by_address(jump_address);

                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Eq, val, zero, op_type);
                            builder.jumpif(cmp, jump_block, Vec::new(), next_block, Vec::new());
                        }
                        Opcode::UBFM => {
                            let positive_condition_block =
                                builder.create_block("ubfm_positive_condition", []);
                            let negative_condition_block =
                                builder.create_block("ubfm_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let immr = Self::get_value(&mut builder, inst.operands[2]);
                            let imms = Self::get_value(&mut builder, inst.operands[3]);
                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Ult, immr, imms, I64);
                            builder.jumpif(
                                cmp,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            let reg_size = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };

                            // copies a bitfield of (<imms>-<immr>+1) bits starting from bit position <immr> in the source register to the least significant bits of the destination register
                            builder.set_insert_block(positive_condition_block);
                            // get src bitfield
                            let one = builder.iconst(1);
                            let src_bitfield_size = builder.add(one, imms, op_type);
                            let src_bitfield_size = builder.sub(src_bitfield_size, immr, op_type);
                            let shift_val = builder.add(imms, one, op_type);
                            let shift_val = builder.sub(reg_size, shift_val, op_type);
                            let val = builder.lshl(src, shift_val, op_type);
                            let shift_val = builder.sub(reg_size, src_bitfield_size, op_type);
                            let val = builder.lshr(val, shift_val, op_type);

                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            builder.set_insert_block(negative_condition_block);
                            let shift_val = builder.add(imms, one, op_type);
                            let shift_val = builder.sub(reg_size, shift_val, op_type);
                            let val = builder.lshl(src, shift_val, op_type);
                            let shift_val = builder.sub(reg_size, immr, op_type);
                            let val = builder.lshr(val, shift_val, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::UDF => {
                            builder.trap();
                        }
                        Opcode::UDIV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = builder.iconst(0);
                            let trap =
                                builder.icmp(tnj::types::cmp::CmpTy::Eq, src2, zero, op_type);
                            builder.trapif(trap);
                            let val = builder.udiv(src1, src2, op_type);
                            Self::write_reg(&mut builder, val, dst_reg, op_type);
                        }
                        Opcode::UMADDL => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let src3 = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.umul(src1, src2, I32);
                            let val = builder.add(val, src3, I64);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        }
                        Opcode::UMSUBL => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let src3 = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.umul(src1, src2, I32);
                            let val = builder.sub(src3, val, I64);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        }
                        Opcode::UMULH => {
                            let dst_reg = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let val = builder.umul(src1, src2, I64);
                            let sixtyfour = builder.iconst(64);
                            let val = builder.ashr(val, sixtyfour, I128);
                            Self::write_reg(&mut builder, val, dst_reg, I64);
                        } // op => unimplemented!("{}", op),
                        _ => {
                            let is_general_purpose =
                                helper::is_operand_general_purpose(inst.operands[0]);
                            if is_general_purpose {
                                let dst_reg = Self::get_dst_reg(&builder, inst);
                                let op_type = helper::get_type_by_inst(inst);
                                let val = builder.opaque(op_type);
                                Self::write_reg(&mut builder, val, dst_reg, op_type);
                            }
                        }
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
                    ShiftStyle::ASR => builder.ashr(reg_val, shift_val, op_type).into(),
                    ShiftStyle::ROR => builder.ror(reg_val, shift_val, op_type).into(),
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
                let rn = Self::reg_val(builder, SizeCode::X, rn, SpOrZrReg::Sp);
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
            _ => builder.opaque(helper::get_type_by_operand(operand)).into(),
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
                    return builder.opaque(op_type).into();
                }
            }
        } else {
            builder.read_reg(Reg(reg as u32), op_type)
        };
        val.into()
    }

    fn get_reg_by_index(builder: &InstructionBuilder, inst: Instruction, index: usize) -> Reg {
        let dst_reg = match inst.operands[index] {
            Operand::Register(_, reg) => Reg(reg as u32),
            Operand::RegisterOrSP(sz, reg) => {
                if reg == 31 {
                    assert_eq!(sz, SizeCode::X, "sp must be 64 bits");
                    builder
                        .get_blob()
                        .get_arch()
                        .lookup_reg(&"sp".into())
                        .unwrap()
                } else {
                    Reg(reg as u32)
                }
            }
            _ => Reg(31),
        };
        dst_reg
    }

    fn get_dst_reg(builder: &InstructionBuilder, inst: Instruction) -> Reg {
        Self::get_reg_by_index(builder, inst, 0)
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
        Self::write_reg(builder, value, reg, BOOL);
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
        let one = builder.iconst(1);
        match operand {
            Operand::ConditionCode(cc) => {
                let inst = match cc {
                    0 => {
                        // EQ
                        let z = Self::flag_value(builder, Flag::Z);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, z, one, BOOL)
                    }
                    1 => {
                        // NE
                        let z = Self::flag_value(builder, Flag::Z);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, z, one, BOOL)
                    }
                    2 => {
                        // CS
                        let c = Self::flag_value(builder, Flag::C);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, c, one, BOOL)
                    }
                    3 => {
                        // CC
                        let c = Self::flag_value(builder, Flag::C);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, c, one, BOOL)
                    }
                    4 => {
                        // MI
                        let n = Self::flag_value(builder, Flag::N);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, n, one, BOOL)
                    }
                    5 => {
                        // PL
                        let n = Self::flag_value(builder, Flag::N);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, n, one, BOOL)
                    }
                    6 => {
                        // VS
                        let v = Self::flag_value(builder, Flag::V);
                        builder.icmp(tnj::types::cmp::CmpTy::Eq, v, one, BOOL)
                    }
                    7 => {
                        // VC
                        let v = Self::flag_value(builder, Flag::V);
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, v, one, BOOL)
                    }
                    8 => {
                        // HI
                        let z = Self::flag_value(builder, Flag::Z);
                        let c = Self::flag_value(builder, Flag::C);

                        let c_is_true = builder.icmp(tnj::types::cmp::CmpTy::Eq, c, one, BOOL);
                        let z_is_false = builder.icmp(tnj::types::cmp::CmpTy::Ne, z, one, BOOL);
                        builder.and(c_is_true, z_is_false, BOOL)
                    }
                    9 => {
                        // LS
                        let z = Self::flag_value(builder, Flag::Z);
                        let c = Self::flag_value(builder, Flag::C);

                        let c_is_false: Inst =
                            builder.icmp(tnj::types::cmp::CmpTy::Ne, c, one, BOOL);

                        let z_is_true = builder.icmp(tnj::types::cmp::CmpTy::Eq, z, one, BOOL);
                        builder.or(c_is_false, z_is_true, BOOL)
                    }
                    10 => {
                        // GE
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        builder.icmp(tnj::types::cmp::CmpTy::Eq, n, v, BOOL)
                    }
                    11 => {
                        // LT
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        builder.icmp(tnj::types::cmp::CmpTy::Ne, n, v, BOOL)
                    }
                    12 => {
                        // GT
                        let z = Self::flag_value(builder, Flag::Z);
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        let z_is_false = builder.icmp(tnj::types::cmp::CmpTy::Ne, z, one, BOOL);
                        let n_eq_v = builder.icmp(tnj::types::cmp::CmpTy::Eq, n, v, BOOL);
                        builder.and(z_is_false, n_eq_v, BOOL)
                    }
                    13 => {
                        // LE
                        let z = Self::flag_value(builder, Flag::Z);
                        let n = Self::flag_value(builder, Flag::N);
                        let v = Self::flag_value(builder, Flag::V);

                        let z_is_true = builder.icmp(tnj::types::cmp::CmpTy::Eq, z, one, BOOL);
                        let n_neq_v = builder.icmp(tnj::types::cmp::CmpTy::Ne, n, v, BOOL);
                        builder.or(z_is_true, n_neq_v, BOOL)
                    }
                    14 => {
                        // AL
                        builder.and(one, one, BOOL)
                    }
                    15 => {
                        // NV
                        builder.icmp(tnj::types::cmp::CmpTy::Ne, one, one, BOOL)
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

    fn set_flags_to_value(builder: &mut InstructionBuilder, flag_val: Value, op_type: Type) {
        let zero = builder.iconst(0);
        // set n flag
        let n_mask = builder.iconst(8);
        let n = builder.and(n_mask, flag_val, op_type);
        let n_is_set = builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, n, op_type);
        Self::write_flag(builder, n_is_set.into(), Flag::N);
        // set z flag
        let z_mask = builder.iconst(4);
        let z = builder.and(z_mask, flag_val, op_type);
        let z_is_set = builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, z, op_type);
        Self::write_flag(builder, z_is_set.into(), Flag::Z);
        // set c flag
        let c_mask = builder.iconst(2);
        let c = builder.and(c_mask, flag_val, op_type);
        let c_is_set = builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, c, op_type);
        Self::write_flag(builder, c_is_set.into(), Flag::C);
        // set v flag
        let v_mask = builder.iconst(1);
        let v = builder.and(v_mask, flag_val, op_type);
        let v_is_set = builder.icmp(tnj::types::cmp::CmpTy::Ne, zero, v, op_type);
        Self::write_flag(builder, v_is_set.into(), Flag::V);
    }

    fn set_flags_using_adc(
        builder: &mut InstructionBuilder,
        val1: Value,
        val2: Value,
        op_type: Type,
        carry: Value,
    ) {
        let zero = builder.iconst(0);
        let sum = builder.add(val1, val2, op_type);
        let sum = builder.add(sum, carry, op_type);

        // z is set if equal if both values are equal
        let z = builder.icmp(tnj::types::cmp::CmpTy::Eq, sum, zero, op_type);
        Self::write_flag(builder, z.into(), Flag::Z);
        // n is set if the sum is negative
        let n = builder.icmp(tnj::types::cmp::CmpTy::Slt, sum, zero, op_type);
        Self::write_flag(builder, n.into(), Flag::N);
        // if either operand is greater than the result in an unsigned comparison, the carry is set
        let val1_is_ugt_sum = builder.icmp(tnj::types::cmp::CmpTy::Ugt, val1, sum, op_type);
        let val2_is_ugt_sum = builder.icmp(tnj::types::cmp::CmpTy::Ugt, val2, sum, op_type);
        let c = builder.or(val1_is_ugt_sum, val2_is_ugt_sum, BOOL);
        Self::write_flag(builder, c.into(), Flag::C);
        // v is set if both operands have the same sign and the result has a different sign
        let val1_is_negative = builder.icmp(tnj::types::cmp::CmpTy::Slt, val1, zero, op_type);
        let val2_is_negative = builder.icmp(tnj::types::cmp::CmpTy::Slt, val2, zero, op_type);
        let values_have_same_sign = builder.icmp(
            tnj::types::cmp::CmpTy::Eq,
            val1_is_negative,
            val2_is_negative,
            BOOL,
        );
        let result_has_different_sign =
            builder.icmp(tnj::types::cmp::CmpTy::Ne, val1_is_negative, n, BOOL);
        let v = builder.and(values_have_same_sign, result_has_different_sign, BOOL);
        Self::write_flag(builder, v.into(), Flag::V);
    }

    // Self::write_reg(&mut builder, val, dst_reg, I64);
    fn write_reg(
        builder: &mut InstructionBuilder,
        val: impl Into<Value>,
        dst_reg: Reg,
        op_type: Type,
    ) {
        if dst_reg.0 != 31 {
            builder.write_reg(val, dst_reg, op_type);
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

/// Error type for disassembling from machine code to AIR
#[derive(Debug, Error)]
pub enum AArch64DisassemblerError {
    /// Error decoding the instructions
    #[error("Error decoding machine code: {0}")]
    DecodeError(#[from] DecodeError),

    /// I/O error
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
