use crate::arm64::helper;
use crate::Lifter;
use target_lexicon::{Aarch64Architecture, Architecture};
use thiserror::Error;
use tnj::air::instructions::builder::InstructionBuilder;
use tnj::air::instructions::{Blob, BlockParamData, Inst, Value};
use tnj::arch::get_arch;
use tnj::arch::reg::Reg;
use tnj::types::{Type, BOOL, I16, I32, I64, I8};
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

// TODO: Implement comparison instruction that also set flags
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
                        builder.set_insert_block(*block);
                    }

                    println!("{}", inst);
                    match inst.opcode {
                        Opcode::ADC | Opcode::ADCS => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let carry = Self::flag_value(&mut builder, Flag::C);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.add(src1, carry, op_type);
                            let val = builder.add(val, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);

                            if inst.opcode == Opcode::ADCS {}
                        }
                        Opcode::ADD => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.add(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ADDS => {
                            // TODO
                        }
                        Opcode::ADR => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let pc = Self::get_pc(&mut builder);
                            let offset = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.add(pc, offset, I64);
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::ADRP => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let offset = Self::get_value(&mut builder, inst.operands[1]);
                            let pc = Self::get_pc(&mut builder);
                            let addr = builder.add(pc, offset, I64);
                            builder.write_reg(addr, dst_reg, I64);
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
                        Opcode::BFM => {
                            let positive_condition_block = builder.create_block(
                                "bfm_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "bfm_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
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
                            let dst_mask = builder.not(dst_mask, op_type);
                            let dst_bitfield = builder.and(src, dst_mask, op_type);
                            // merge and write bitfield
                            let val = builder.or(src_bitfield, dst_bitfield, op_type);
                            builder.write_reg(val, dst_reg, op_type);
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
                            let dst_mask = builder.not(dst_mask, op_type);
                            let dst_bitfield = builder.and(src, dst_mask, op_type);
                            // merge and write bitfield
                            let val = builder.or(src_bitfield, dst_bitfield, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::BIC => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let neg_src2 = builder.not(src2, op_type);
                            let val = builder.and(src1, neg_src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::BL => {
                            let instruction_size = builder.iconst(4);
                            let pc_reg = Self::get_pc(&mut builder);
                            let return_address = builder.add(pc_reg, instruction_size, I64);
                            let x30 = Self::get_reg_val_by_name(&mut builder, "x30");

                            let offset = helper::get_pc_offset_as_int(inst.operands[0]);
                            let next_address = pc.wrapping_add(offset);
                            let block = label_resolver.get_block_by_address(next_address);
                            builder.write_reg(return_address, x30, I64);
                            builder.jump(*block, vec![]);
                        }
                        Opcode::BLR => {
                            unimplemented!("BLR. Need to implement jump to register")
                        }
                        Opcode::BR => {
                            unimplemented!("BR. Need to implement jump to register")
                        }
                        Opcode::CAS(_memory_ordering) => {
                            // Untested
                            let swap_block =
                                builder.create_block("cas_swap", Vec::<BlockParamData>::new());
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let old = Self::get_value(&mut builder, inst.operands[0]);
                            let new = Self::get_value(&mut builder, inst.operands[1]);
                            let addr = Self::get_value(&mut builder, inst.operands[2]);
                            let sz = Self::get_size_code(inst.operands[0]);
                            let op_type = helper::get_type_by_sizecode(sz);
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
                            let op_type =
                                helper::get_type_by_sizecode(Self::get_size_code(inst.operands[0]));
                            let zero = builder.iconst(0);
                            let condition =
                                builder.icmp(tnj::types::cmp::CmpTy::Ne, src, zero, op_type);

                            let offset = helper::get_pc_offset_as_int(inst.operands[1]);
                            let jump_address = pc.wrapping_add(offset);
                            let block = label_resolver.get_block_by_address(jump_address);

                            builder.jumpif(condition, *block, Vec::new(), next_block, Vec::new());
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
                            let jump_address = pc.wrapping_add(offset);
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
                            let flag_val = Self::get_value(&mut builder, inst.operands[2]);
                            Self::set_flags_to_value(&mut builder, flag_val, op_type);
                            builder.jump(next_block, Vec::new());

                            builder.set_insert_block(negative_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[0]);
                            let src2 = Self::get_value(&mut builder, inst.operands[1]);
                            Self::set_flags_using_comparison(&mut builder, src1, src2, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CCMN => {
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
                            let flag_val = Self::get_value(&mut builder, inst.operands[2]);
                            Self::set_flags_to_value(&mut builder, flag_val, op_type);
                            builder.jump(next_block, Vec::new());

                            builder.set_insert_block(negative_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[0]);
                            let zero = builder.iconst(0);
                            let src2 = Self::get_value(&mut builder, inst.operands[1]);
                            let neg_src2 = builder.sub(zero, src2, op_type);
                            Self::set_flags_using_comparison(
                                &mut builder,
                                src1,
                                neg_src2.into(),
                                op_type,
                            );
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CLS => {
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);

                            let one = builder.iconst(1);
                            let first_bit_clear_mask = builder.not(one, op_type);
                            let val1 = builder.and(src, first_bit_clear_mask, op_type);
                            let val2 = builder.lshl(src, one, op_type);
                            let val = builder.xor(val1, val2, op_type);

                            let n = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };
                            let highest_set_bit = builder.highest_set_bit(val, op_type);
                            let val = builder.sub(n, highest_set_bit, op_type);
                            let val = builder.sub(val, one, op_type);

                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::CLZ => {
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let one = builder.iconst(1);

                            let n = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };
                            let highest_set_bit = builder.highest_set_bit(src, op_type);
                            let val = builder.sub(n, highest_set_bit, op_type);
                            let val = builder.sub(val, one, op_type);

                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::CSEL => {
                            let positive_condition_block = builder.create_block(
                                "csel_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "csel_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

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

                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            builder.write_reg(src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            builder.set_insert_block(negative_condition_block);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            builder.write_reg(src2, dst_reg, op_type);
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
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let val = builder.add(src2, one, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            builder.write_reg(src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSINV => {
                            let positive_condition_block = builder.create_block(
                                "csinv_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "csinv_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

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
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let val = builder.not(src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            builder.write_reg(src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSNEG => {
                            let positive_condition_block = builder.create_block(
                                "csneg_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "csneg_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

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
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let zero = builder.iconst(0);
                            let val = builder.sub(zero, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            builder.write_reg(src1, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::EON => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);

                            let src2 = builder.not(src2, op_type);
                            let val = builder.xor(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::EOR => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            builder.xor(src1, src2, op_type);
                            builder.write_reg(src1, dst_reg, op_type);
                        }
                        Opcode::LDP | Opcode::LDXP => {
                            let (dst_reg1, sz) = Self::get_reg_by_index(&builder, inst, 0);
                            let (dst_reg2, _) = Self::get_reg_by_index(&builder, inst, 1);
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
                        Opcode::EXTR => {
                            // 4 Operands
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
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
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::HINT => {
                            // HINT is a no-op
                        }
                        Opcode::HVC => {
                            // TODO
                            unimplemented!("HVC");
                        }
                        Opcode::LDR | Opcode::LDUR | Opcode::LDAR | Opcode::LDXR => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::LDRB | Opcode::LDURB | Opcode::LDARB | Opcode::LDXRB => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, I8);
                            let val = builder.zext_i8(val, I32);
                            builder.write_reg(val, dst_reg, I32);
                        }
                        Opcode::LDRH | Opcode::LDURH | Opcode::LDARH | Opcode::LDXRH => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.load(address, I16);
                            let val = builder.zext_i16(val, I32);
                            builder.write_reg(val, dst_reg, I32);
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
                        Opcode::MOVK => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            builder.write_reg(src, dst_reg, I16);
                        }
                        Opcode::MOVN => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let zero = builder.iconst(0);
                            builder.write_reg(zero, dst_reg, op_type);

                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let src = builder.not(src, I16);
                            builder.write_reg(src, dst_reg, I16);
                        }
                        Opcode::MOVZ => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let zero = builder.iconst(0);
                            builder.write_reg(zero, dst_reg, op_type);

                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            builder.write_reg(src, dst_reg, I16);
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
                        Opcode::RBIT => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.reverse_bits(src, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::RET => {
                            builder.ret();
                        }
                        Opcode::RETAB => {
                            // TODO
                        }
                        Opcode::REV | Opcode::REV64 => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let src = Self::get_value(&mut builder, inst.operands[1]);
                            let val = builder.reverse_bytes(src, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::REV16 => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let (src_reg, _) = Self::get_reg_by_index(&builder, inst, 1);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let mut res = builder.iconst(0);
                            let sixteen = builder.iconst(16);

                            let loop_iterations = if sz == SizeCode::W { 2 } else { 4 };
                            for i in 0..loop_iterations {
                                let val = builder.read_reg(src_reg, I16);
                                let val = builder.reverse_bytes(val, I16);
                                res = builder.or(res, val, I16).into();
                                if i != loop_iterations - 1 {
                                    res = builder.lshl(res, sixteen, I32).into();
                                }
                            }
                            builder.write_reg(res, dst_reg, op_type);
                        }
                        Opcode::REV32 => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let (src_reg, _) = Self::get_reg_by_index(&builder, inst, 1);
                            let mut res = builder.iconst(0);

                            let val = builder.read_reg(src_reg, I32);
                            let val = builder.reverse_bytes(val, I32);
                            res = builder.or(res, val, I32).into();

                            let thirtytwo = builder.iconst(32);
                            builder.lshl(res, thirtytwo, I64);
                            let val = builder.read_reg(src_reg, I32);
                            let val = builder.reverse_bytes(val, I32);
                            res = builder.or(res, val, I32).into();

                            builder.write_reg(res, dst_reg, I64);
                        }
                        Opcode::RORV => {
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let size = match op_type {
                                I64 => builder.iconst(64),
                                _ => builder.iconst(32),
                            };
                            let shift = builder.modulo(src2, size, op_type);
                            let val = builder.ror(src1, shift, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::SBC => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let carry = Self::flag_value(&mut builder, Flag::C);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.sub(src1, src2, op_type);
                            let val = builder.sub(val, carry, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::SBCS => {
                            // TODO
                        }
                        Opcode::SBFM => {
                            let positive_condition_block = builder.create_block(
                                "sbfm_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "sbfm_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
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
                            let val = builder.ashr(val, shift_val, op_type);

                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            builder.set_insert_block(negative_condition_block);
                            let shift_val = builder.add(imms, one, op_type);
                            let shift_val = builder.sub(reg_size, shift_val, op_type);
                            let val = builder.lshl(src, shift_val, op_type);
                            let shift_val = builder.sub(reg_size, immr, op_type);
                            let val = builder.ashr(val, shift_val, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::SDIV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let zero = builder.iconst(0);
                            let trap =
                                builder.icmp(tnj::types::cmp::CmpTy::Eq, src2, zero, op_type);
                            builder.trapif(trap, op_type);
                            let val = builder.idiv(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::SMADDL => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let src3 = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(src1, src2, I32);
                            let val = builder.add(val, src3, I64);
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::SMSUBL => {
                            let (dst_reg, _) = Self::get_dst_reg(&builder, inst);
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let src3 = Self::get_value(&mut builder, inst.operands[3]);
                            let val = builder.imul(src1, src2, I32);
                            let val = builder.sub(src3, val, I64);
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::SMULH => {
                            // TODO
                        }
                        Opcode::STP | Opcode::STLXP | Opcode::STXP | Opcode::STNP => {
                            let src1 = Self::get_value(&mut builder, inst.operands[0]);
                            let src2 = Self::get_value(&mut builder, inst.operands[1]);
                            let address = Self::get_value(&mut builder, inst.operands[2]);
                            let size_code = Self::get_size_code(inst.operands[0]);
                            let op_type = helper::get_type_by_sizecode(size_code);

                            builder.store(src1, address, op_type);
                            let address_offset = match op_type {
                                I64 => builder.iconst(8),
                                _ => builder.iconst(4),
                            };
                            let address = builder.add(address, address_offset, I64);
                            builder.store(src2, address, op_type);
                        }
                        Opcode::STR
                        | Opcode::STLR
                        | Opcode::STLXR
                        | Opcode::STUR
                        | Opcode::STLUR
                        | Opcode::STXR => {
                            let sz = Self::get_size_code(inst.operands[0]);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let value = Self::get_value(&mut builder, inst.operands[0]);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            builder.store(value, address, op_type);
                        }
                        Opcode::STRB
                        | Opcode::STLRB
                        | Opcode::STLXRB
                        | Opcode::STURB
                        | Opcode::STLURB
                        | Opcode::STXRB => {
                            let value = Self::get_value(&mut builder, inst.operands[0]);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            builder.store(value, address, I8);
                        }
                        Opcode::STRH
                        | Opcode::STLRH
                        | Opcode::STLXRH
                        | Opcode::STURH
                        | Opcode::STLURH
                        | Opcode::STXRH => {
                            let value = Self::get_value(&mut builder, inst.operands[0]);
                            let address = Self::get_value(&mut builder, inst.operands[1]);
                            builder.store(value, address, I32);
                        }
                        Opcode::SUB => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let val = builder.sub(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::SUBS => {
                            // TODO
                        }
                        Opcode::SVC => {
                            // TODO
                        }
                        Opcode::SYS(_data) => {
                            // TODO
                        }
                        Opcode::SYSL(_data) => {
                            // TODO
                        }
                        Opcode::TBNZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let one = builder.iconst(1);
                            let zero = builder.iconst(0);
                            let (src, sizecode) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sizecode);
                            let test_bit = Self::get_value(&mut builder, inst.operands[1]);
                            let offset = helper::get_pc_offset_as_int(inst.operands[2]);

                            let test_bit = builder.lshr(test_bit, one, op_type);
                            let val = builder.and(test_bit, src, op_type);
                            let jump_address = pc.wrapping_add(offset);
                            let jump_block = *label_resolver.get_block_by_address(jump_address);

                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Ne, val, zero, op_type);
                            builder.jumpif(cmp, jump_block, Vec::new(), next_block, Vec::new());
                        }
                        Opcode::TBZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let one = builder.iconst(1);
                            let zero = builder.iconst(0);
                            let (src, sizecode) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sizecode);
                            let test_bit = Self::get_value(&mut builder, inst.operands[1]);
                            let offset = helper::get_pc_offset_as_int(inst.operands[2]);

                            let test_bit = builder.lshr(test_bit, one, op_type);
                            let val = builder.and(test_bit, src, op_type);
                            let jump_address = pc.wrapping_add(offset);
                            let jump_block = *label_resolver.get_block_by_address(jump_address);

                            let cmp = builder.icmp(tnj::types::cmp::CmpTy::Eq, val, zero, op_type);
                            builder.jumpif(cmp, jump_block, Vec::new(), next_block, Vec::new());
                        }
                        Opcode::UBFM => {
                            let positive_condition_block = builder.create_block(
                                "ubfm_positive_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let negative_condition_block = builder.create_block(
                                "ubfm_negative_condition",
                                Vec::<BlockParamData>::new(),
                            );
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = *label_resolver.get_block_by_address(next_address);

                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
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

                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            builder.set_insert_block(negative_condition_block);
                            let shift_val = builder.add(imms, one, op_type);
                            let shift_val = builder.sub(reg_size, shift_val, op_type);
                            let val = builder.lshl(src, shift_val, op_type);
                            let shift_val = builder.sub(reg_size, immr, op_type);
                            let val = builder.lshr(val, shift_val, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                            builder.jump(next_block, Vec::new());
                        }
                        Opcode::UDIV => {
                            let src1 = Self::get_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let op_type = helper::get_type_by_sizecode(sz);
                            let zero = builder.iconst(0);
                            let trap =
                                builder.icmp(tnj::types::cmp::CmpTy::Eq, src2, zero, op_type);
                            builder.trapif(trap, op_type);
                            let val = builder.udiv(src1, src2, op_type);
                            builder.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::UMADDL => {
                            // TODO
                        }
                        Opcode::UMSUBL => {
                            // TODO
                        }
                        Opcode::UMULH => {
                            // TODO
                        }
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
                    ShiftStyle::ASR => builder.ashr(reg_val, shift_val, op_type).into(),
                    ShiftStyle::ROR => unimplemented!("ROR"), // TODO
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

    fn get_reg_by_index(
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

    fn set_flags_using_comparison(
        builder: &mut InstructionBuilder,
        val1: Value,
        val2: Value,
        op_type: Type,
    ) {
        let zero = builder.iconst(0);
        let one = builder.iconst(1);
        let val2 = builder.not(val2, op_type);
        let sum = builder.add(val1, val2, op_type);
        let sum = builder.add(sum, one, op_type);

        // z is set if equal if both values are equal
        let z = builder.icmp(tnj::types::cmp::CmpTy::Eq, sum, zero, op_type);
        Self::write_flag(builder, z.into(), Flag::Z);
        // n is set if the sum is negative
        let n = builder.icmp(tnj::types::cmp::CmpTy::Slt, sum, zero, op_type);
        Self::write_flag(builder, n.into(), Flag::N);
        // c is set if operation creates carry
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
