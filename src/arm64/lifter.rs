use crate::arm64::{helper, LabelResolver};
use crate::Lifter;
use target_lexicon::{Aarch64Architecture, Architecture};
use thiserror::Error;
use tnj::air::instructions::builder::InstructionBuilder;
use tnj::air::instructions::{CodeRegion, Inst, Value};
use tnj::arch::get_arch;
use tnj::arch::reg::Reg;
use tnj::types::cmp::CmpTy;
use tnj::types::{Type, BOOL, I128, I16, I32, I64, I8};
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{
    ARMv8, DecodeError, InstDecoder, Instruction, Opcode, Operand, ShiftStyle, SizeCode,
};

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

    fn lift(&self, code: &[u8], _proofs: &[u8]) -> Result<CodeRegion, Self::E> {
        let arch = get_arch(Architecture::Aarch64(Aarch64Architecture::Aarch64)).unwrap();
        let mut code_region = CodeRegion::new(arch);

        let state = LifterState::new(&mut code_region, code)?;

        state.lift()?;

        Ok(code_region)
    }
}

/// Private lifter tate
struct LifterState<'a> {
    builder: InstructionBuilder<'a>,
    label_resolver: LabelResolver,
    decoder: InstDecoder,
    reader: U8Reader<'a>,
}

impl<'a> LifterState<'a> {
    fn new(code_region: &'a mut CodeRegion, code: &'a [u8]) -> Result<Self, AArch64LifterError> {
        let builder = code_region.insert();
        let decoder = <ARMv8 as Arch>::Decoder::default();
        let reader = U8Reader::new(code);
        let label_resolver = LabelResolver::new(code, &decoder)?;

        Ok(Self {
            builder,
            label_resolver,
            decoder,
            reader,
        })
    }

    fn lift(mut self) -> Result<(), AArch64LifterError> {
        self.label_resolver.create_blocks(&mut self.builder);

        let mut pc = 0u64;

        loop {
            match self.decoder.decode(&mut self.reader) {
                Ok(inst) => {
                    let block = self.label_resolver.get_block(pc);
                    if let Some(block) = block {
                        self.builder.jump(block, vec![]);
                        self.builder.set_insert_block(block);
                    }

                    match inst.opcode {
                        Opcode::ADC | Opcode::ADCS => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let carry = self.flag_value(Flag::C);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.add(src1, carry, op_type);
                            let val = self.builder.add(val, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);

                            if inst.opcode == Opcode::ADCS {
                                self.set_flags_using_adc(src1, src2, op_type, carry);
                            }
                        }
                        Opcode::ADD | Opcode::ADDS => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.add(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);

                            if inst.opcode == Opcode::ADDS {
                                let zero = self.builder.iconst(0);
                                self.set_flags_using_adc(src1, src2, op_type, zero);
                            }
                        }
                        Opcode::ADR => {
                            let dst_reg = self.get_dst_reg(inst);
                            let pc = self.read_pc_reg();
                            let offset = self.get_value(inst.operands[1]);
                            let val = self.builder.add(pc, offset, I64);
                            self.write_reg(val, dst_reg, I64);
                        }
                        Opcode::ADRP => {
                            let dst_reg = self.get_dst_reg(inst);
                            let offset = self.get_value(inst.operands[1]);
                            let reverse_mask = self.builder.iconst(0xFFF);
                            let mask = self.builder.bitwise_not(reverse_mask, I64);
                            let pc = self.read_pc_reg();
                            let masked_pc = self.builder.and(pc, mask, I64);
                            let addr = self.builder.add(masked_pc, offset, I64);
                            self.write_reg(addr, dst_reg, I64);
                        }
                        Opcode::AND | Opcode::ANDS => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.and(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);

                            if inst.opcode == Opcode::ANDS {
                                let zero = self.builder.iconst(0);
                                self.write_flag(zero, Flag::C);
                                self.write_flag(zero, Flag::V);
                                let is_zero = self.builder.icmp(CmpTy::Eq, val, zero, op_type);
                                self.write_flag(is_zero.into(), Flag::Z);
                                let is_negative = self.builder.icmp(CmpTy::Slt, val, zero, op_type);
                                self.write_flag(is_negative.into(), Flag::N);
                            }
                        }
                        Opcode::ASRV => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let shift_mask = match op_type {
                                I64 => self.builder.iconst(63),
                                _ => self.builder.iconst(31),
                            };
                            let shift_val = self.builder.and(src2, shift_mask, op_type);
                            let val = self.builder.ashr(src1, shift_val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::B | Opcode::BL => {
                            if inst.opcode == Opcode::BL {
                                let instruction_size = self.builder.iconst(4);
                                let pc_reg = self.read_pc_reg();
                                let return_address =
                                    self.builder.add(pc_reg, instruction_size, I64);
                                let x30 = self.get_reg_val_by_name("x30");
                                self.write_reg(return_address, x30, I64);
                            }
                            let offset = helper::get_pc_offset_as_int(inst.operands[0]);
                            let next_address = (pc as i64).wrapping_add(offset) as u64;
                            let block = self.label_resolver.get_block(next_address).unwrap();
                            self.builder.jump(block, vec![]);
                        }
                        Opcode::Bcc(condition) => {
                            let offset = helper::get_pc_offset_as_int(inst.operands[0]);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let jump_block = self.label_resolver.get_block(jump_address).unwrap();
                            let next_address: u64 = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let operand = Operand::ConditionCode(condition);
                            let condition = self.get_condition(operand)?;
                            self.builder.jumpif(
                                condition,
                                jump_block,
                                Vec::new(),
                                next_block,
                                Vec::new(),
                            );
                        }
                        Opcode::BFM => {
                            let positive_condition_block =
                                self.builder.create_block("bfm_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("bfm_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = self.get_value(inst.operands[1]);
                            let immr = self.get_value(inst.operands[2]);
                            let imms = self.get_value(inst.operands[3]);
                            let cmp = self.builder.icmp(CmpTy::Uge, imms, immr, I64);
                            self.builder.jumpif(
                                cmp,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // copies a bitfield of (<imms>-<immr>+1) bits starting from bit position <immr> in the source register to the least significant bits of the destination register
                            self.builder.set_insert_block(positive_condition_block);
                            // get src bitfield
                            let one = self.builder.iconst(1);
                            let src_bitfield_size = self.builder.add(one, imms, op_type);
                            let src_bitfield_size =
                                self.builder.sub(src_bitfield_size, immr, op_type);
                            let src_mask = self.builder.lshl(one, src_bitfield_size, op_type);
                            let src_mask = self.builder.sub(src_mask, one, op_type);
                            let src_mask = self.builder.lshl(src_mask, immr, op_type);
                            let src_bitfield = self.builder.and(src, src_mask, op_type);
                            let src_bitfield = self.builder.lshr(src_bitfield, immr, op_type);
                            // clear dst bits that are replaced by the src bitfield
                            let dst_mask = self.builder.lshl(one, src_bitfield_size, op_type);
                            let dst_mask = self.builder.sub(dst_mask, one, op_type);
                            let dst_mask = self.builder.bitwise_not(dst_mask, op_type);
                            let dst_bitfield = self.builder.and(src, dst_mask, op_type);
                            // merge and write bitfield
                            let val = self.builder.or(src_bitfield, dst_bitfield, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            self.builder.set_insert_block(negative_condition_block);
                            // get bitfield containing src bits
                            let src_bitfield_size = self.builder.add(one, imms, op_type);
                            let src_mask = self.builder.lshl(one, src_bitfield_size, op_type);
                            let src_mask = self.builder.sub(src_mask, one, op_type);
                            let src_bitfield = self.builder.and(src, src_mask, op_type);
                            let reg_size = match op_type {
                                I64 => self.builder.iconst(64),
                                _ => self.builder.iconst(32),
                            };
                            let starting_position = self.builder.sub(reg_size, immr, op_type);
                            let src_bitfield =
                                self.builder.lshl(src_bitfield, starting_position, op_type);
                            // clear dst bits that are replaced by the src bitfield
                            let dst_mask = self.builder.lshl(one, src_bitfield_size, op_type);
                            let dst_mask = self.builder.sub(dst_mask, one, op_type);
                            let dst_mask = self.builder.lshl(dst_mask, starting_position, op_type);
                            let dst_mask = self.builder.bitwise_not(dst_mask, op_type);
                            let dst_bitfield = self.builder.and(src, dst_mask, op_type);
                            // merge and write bitfield
                            let val = self.builder.or(src_bitfield, dst_bitfield, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::BIC => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let neg_src2 = self.builder.bitwise_not(src2, op_type);
                            let val = self.builder.and(src1, neg_src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::BLR | Opcode::BR => {
                            if inst.opcode == Opcode::BLR {
                                let pc = self.read_pc_reg();
                                let four = self.builder.iconst(4);
                                let ret_address = self.builder.add(pc, four, I64);
                                let x30 = self.get_reg_val_by_name("x30");
                                self.write_reg(ret_address, x30, I64);
                            }
                            let address = self.get_value(inst.operands[0]);
                            self.builder.dynamic_jump(address);
                            if inst.opcode == Opcode::BLR {
                                self.mark_next_block_as_entry(pc);
                            }
                        }
                        Opcode::CAS(_memory_ordering) => {
                            // Untested
                            let swap_block = self.builder.create_block("cas_swap", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let old = self.get_value(inst.operands[0]);
                            let new = self.get_value(inst.operands[1]);
                            let addr = self.get_value(inst.operands[2]);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.load(addr, op_type);
                            let cmp = self.builder.icmp(CmpTy::Eq, val, old, op_type);
                            self.builder.jumpif(
                                cmp,
                                swap_block,
                                Vec::new(),
                                next_block,
                                Vec::new(),
                            );

                            self.builder.set_insert_block(swap_block);
                            self.builder.store(new, addr, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::CBNZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let src = self.get_value(inst.operands[0]);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = self.builder.iconst(0);
                            let condition = self.builder.icmp(CmpTy::Ne, src, zero, op_type);

                            let offset = helper::get_pc_offset_as_int(inst.operands[1]);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let block = self.label_resolver.get_block(jump_address).unwrap();

                            self.builder.jumpif(
                                condition,
                                block,
                                Vec::new(),
                                next_block,
                                Vec::new(),
                            );
                        }
                        Opcode::CBZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let src = self.get_value(inst.operands[0]);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = self.builder.iconst(0);
                            let condition = self.builder.icmp(CmpTy::Eq, src, zero, op_type);

                            let offset = helper::get_pc_offset_as_int(inst.operands[1]);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let block = self.label_resolver.get_block(jump_address).unwrap();

                            self.builder.jumpif(
                                condition,
                                block,
                                Vec::new(),
                                next_block,
                                Vec::new(),
                            );
                        }
                        Opcode::CCMN => {
                            let positive_condition_block =
                                self.builder.create_block("ccmp_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("ccmp_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let condition = self.get_condition(inst.operands[3])?;
                            let op_type = helper::get_type_by_inst(inst);
                            self.builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            self.builder.set_insert_block(positive_condition_block);
                            let src1 = self.get_value(inst.operands[0]);
                            let src2 = self.get_value(inst.operands[1]);
                            let carry = self.builder.iconst(0);
                            self.set_flags_using_adc(src1, src2, op_type, carry);
                            self.builder.jump(next_block, Vec::new());

                            self.builder.set_insert_block(negative_condition_block);
                            let flag_val = self.get_value(inst.operands[2]);
                            self.set_flags_to_value(flag_val, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::CCMP => {
                            let positive_condition_block =
                                self.builder.create_block("ccmp_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("ccmp_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let condition = self.get_condition(inst.operands[3])?;
                            let op_type = helper::get_type_by_inst(inst);
                            self.builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            self.builder.set_insert_block(positive_condition_block);
                            let src1 = self.get_value(inst.operands[0]);
                            let src2 = self.get_value(inst.operands[1]);
                            let not_src2 = self.builder.bitwise_not(src2, op_type);
                            let carry = self.builder.iconst(0);
                            self.set_flags_using_adc(src1, not_src2.into(), op_type, carry);
                            self.builder.jump(next_block, Vec::new());

                            self.builder.set_insert_block(negative_condition_block);
                            let flag_val = self.get_value(inst.operands[2]);
                            self.set_flags_to_value(flag_val, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::CLS => {
                            let src = self.get_value(inst.operands[1]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);

                            let one = self.builder.iconst(1);
                            let val1 = self.builder.lshr(src, one, op_type);
                            let val2_mask = self.builder.ror(one, one, op_type);
                            let val2_mask = self.builder.bitwise_not(val2_mask, op_type);
                            let val2 = self.builder.and(val2_mask, src, op_type);
                            let val = self.builder.xor(val1, val2, op_type);

                            let n = match op_type {
                                I64 => self.builder.iconst(64),
                                _ => self.builder.iconst(32),
                            };
                            let highest_set_bit = self.builder.highest_set_bit(val, op_type);
                            let val = self.builder.sub(n, highest_set_bit, op_type);
                            let val = self.builder.sub(val, one, op_type);

                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::CLZ => {
                            let src = self.get_value(inst.operands[1]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let one = self.builder.iconst(1);

                            let n = match op_type {
                                I64 => self.builder.iconst(64),
                                _ => self.builder.iconst(32),
                            };
                            let highest_set_bit = self.builder.highest_set_bit(src, op_type);
                            let val = self.builder.sub(n, highest_set_bit, op_type);
                            let val = self.builder.sub(val, one, op_type);

                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::CSEL => {
                            let positive_condition_block =
                                self.builder.create_block("csel_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("csel_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = self.get_condition(inst.operands[3])?;
                            self.builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            self.builder.set_insert_block(positive_condition_block);
                            let src1 = self.get_value(inst.operands[1]);
                            self.write_reg(src1, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());

                            self.builder.set_insert_block(negative_condition_block);
                            let src2 = self.get_value(inst.operands[2]);
                            self.write_reg(src2, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSINC => {
                            let positive_condition_block =
                                self.builder.create_block("csinc_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("csinc_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = self.get_condition(inst.operands[3])?;
                            self.builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // Condition is true
                            self.builder.set_insert_block(positive_condition_block);
                            let src1 = self.get_value(inst.operands[1]);
                            self.write_reg(src1, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());

                            // Condition is false
                            self.builder.set_insert_block(negative_condition_block);
                            let one = self.builder.iconst(1);
                            let src2 = self.get_value(inst.operands[2]);
                            let val = self.builder.add(src2, one, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSINV => {
                            let positive_condition_block =
                                self.builder.create_block("csinv_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("csinv_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = self.get_condition(inst.operands[3])?;
                            self.builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // Condition is true
                            self.builder.set_insert_block(positive_condition_block);
                            let src1 = self.get_value(inst.operands[1]);
                            self.write_reg(src1, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());

                            // Condition is false
                            self.builder.set_insert_block(negative_condition_block);
                            let src2 = self.get_value(inst.operands[2]);
                            let val = self.builder.bitwise_not(src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::CSNEG => {
                            let positive_condition_block =
                                self.builder.create_block("csneg_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("csneg_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let condition = self.get_condition(inst.operands[3])?;
                            self.builder.jumpif(
                                condition,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            // Condition is true
                            self.builder.set_insert_block(positive_condition_block);
                            let src1 = self.get_value(inst.operands[1]);
                            self.write_reg(src1, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());

                            // Condition is false
                            self.builder.set_insert_block(negative_condition_block);
                            let src2 = self.get_value(inst.operands[2]);
                            let zero = self.builder.iconst(0);
                            let val = self.builder.sub(zero, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::EON => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);

                            let src2 = self.builder.bitwise_not(src2, op_type);
                            let val = self.builder.xor(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::EOR => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            self.builder.xor(src1, src2, op_type);
                            self.write_reg(src1, dst_reg, op_type);
                        }
                        Opcode::EXTR => {
                            // 4 Operands
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let shift_val = self.get_value(inst.operands[3]);

                            let datasize = match op_type {
                                I64 => self.builder.iconst(64),
                                _ => self.builder.iconst(32),
                            };
                            let src2 = self.builder.lshr(src2, shift_val, op_type);
                            let shift_val = self.builder.sub(datasize, shift_val, op_type);
                            let src1 = self.builder.lshl(src1, shift_val, op_type);
                            let val = self.builder.or(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::HINT => {
                            // HINT is a no-op
                        }
                        Opcode::HVC => {
                            // We are ignoring hypervisor calls
                            self.mark_next_block_as_entry(pc);
                        }
                        Opcode::LDP | Opcode::LDXP => {
                            let dst_reg1 = self.get_reg_by_index(inst, 0);
                            let dst_reg2 = self.get_reg_by_index(inst, 1);
                            let address = self.get_value(inst.operands[2]);
                            let op_type = helper::get_type_by_inst(inst);

                            let val1 = self.builder.load(address, op_type);
                            self.write_reg(val1, dst_reg1, op_type);
                            let address_offset = match op_type {
                                I64 => self.builder.iconst(8),
                                _ => self.builder.iconst(4),
                            };
                            let address = self.builder.add(address, address_offset, I64);
                            let val2 = self.builder.load(address, op_type);
                            self.write_reg(val2, dst_reg2, op_type);
                        }
                        Opcode::LDPSW => {
                            let dst_reg1 = self.get_reg_by_index(inst, 0);
                            let dst_reg2 = self.get_reg_by_index(inst, 1);
                            let address = self.get_value(inst.operands[2]);

                            let val1 = self.builder.load(address, I32);
                            let val1 = self.builder.sext_i32(val1, I64);
                            self.write_reg(val1, dst_reg1, I64);
                            let address_offset = self.builder.iconst(4);
                            let address = self.builder.add(address, address_offset, I64);
                            let val2 = self.builder.load(address, I32);
                            let val2 = self.builder.sext_i32(val2, I64);
                            self.write_reg(val2, dst_reg2, I64);
                        }
                        Opcode::LDR
                        | Opcode::LDUR
                        | Opcode::LDAR
                        | Opcode::LDXR
                        | Opcode::LDAXR
                        | Opcode::LDTR => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let address = self.get_value(inst.operands[1]);
                            let val = self.builder.load(address, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::LDRB
                        | Opcode::LDURB
                        | Opcode::LDARB
                        | Opcode::LDXRB
                        | Opcode::LDAXRB
                        | Opcode::LDTRB => {
                            let dst_reg = self.get_dst_reg(inst);
                            let address = self.get_value(inst.operands[1]);
                            let val = self.builder.load(address, I8);
                            let val = self.builder.zext_i8(val, I32);
                            self.write_reg(val, dst_reg, I32);
                        }
                        Opcode::LDRH
                        | Opcode::LDURH
                        | Opcode::LDARH
                        | Opcode::LDXRH
                        | Opcode::LDAXRH
                        | Opcode::LDTRH => {
                            let dst_reg = self.get_dst_reg(inst);
                            let address = self.get_value(inst.operands[1]);
                            let val = self.builder.load(address, I16);
                            let val = self.builder.zext_i16(val, I32);
                            self.write_reg(val, dst_reg, I32);
                        }
                        Opcode::LDRSB | Opcode::LDTRSB | Opcode::LDURSB => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let address = self.get_value(inst.operands[1]);
                            let val = self.builder.load(address, I8);
                            let val = self.builder.sext_i8(val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::LDRSH | Opcode::LDTRSH | Opcode::LDURSH => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let address = self.get_value(inst.operands[1]);
                            let val = self.builder.load(address, I16);
                            let val = self.builder.sext_i16(val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::LDRSW | Opcode::LDTRSW | Opcode::LDURSW => {
                            let dst_reg = self.get_dst_reg(inst);
                            let address = self.get_value(inst.operands[1]);
                            let val = self.builder.load(address, I32);
                            let val = self.builder.sext_i32(val, I64);
                            self.write_reg(val, dst_reg, I64);
                        }
                        Opcode::LSLV => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let shift_mask = match op_type {
                                I64 => self.builder.iconst(63),
                                _ => self.builder.iconst(31),
                            };
                            let shift_val = self.builder.and(src2, shift_mask, op_type);
                            let val = self.builder.lshl(src1, shift_val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::LSRV => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let shift_mask = match op_type {
                                I64 => self.builder.iconst(63),
                                _ => self.builder.iconst(31),
                            };
                            let shift_val = self.builder.and(src2, shift_mask, op_type);
                            let val = self.builder.lshr(src1, shift_val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::MADD => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let mul_src1 = self.get_value(inst.operands[1]);
                            let mul_src2 = self.get_value(inst.operands[2]);
                            let add_src = self.get_value(inst.operands[3]);
                            let val = self.builder.imul(mul_src1, mul_src2, op_type);
                            let val = self.builder.add(val, add_src, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::MOVK => {
                            let dst_reg = self.get_dst_reg(inst);
                            let src = self.get_value(inst.operands[1]);
                            self.write_reg(src, dst_reg, I16);
                        }
                        Opcode::MOVN => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = self.builder.iconst(0);
                            self.write_reg(zero, dst_reg, op_type);

                            let src = self.get_value(inst.operands[1]);
                            let src = self.builder.bitwise_not(src, I16);
                            self.write_reg(src, dst_reg, I16);
                        }
                        Opcode::MOVZ => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = self.builder.iconst(0);
                            self.write_reg(zero, dst_reg, op_type);

                            let src = self.get_value(inst.operands[1]);
                            self.write_reg(src, dst_reg, I16);
                        }
                        Opcode::MSUB => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let mul_src1 = self.get_value(inst.operands[1]);
                            let mul_src2 = self.get_value(inst.operands[2]);
                            let sub_src = self.get_value(inst.operands[3]);
                            let val = self.builder.imul(mul_src1, mul_src2, op_type);
                            let val = self.builder.sub(sub_src, val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::NEG => {
                            let zero = self.builder.iconst(0);
                            let src = self.get_value(inst.operands[1]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.sub(zero, src, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ORN => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let val = self.builder.bitwise_not(src2, op_type);
                            let val = self.builder.or(src1, val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::ORR => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.or(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::PRFM | Opcode::PRFUM => {
                            // We are ignoring prefetch hints
                        }
                        Opcode::RBIT => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = self.get_value(inst.operands[1]);
                            let val = self.builder.reverse_bits(src, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::RET | Opcode::RETAB | Opcode::RETAA => {
                            self.builder.ret();
                        }
                        Opcode::REV | Opcode::REV64 => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = self.get_value(inst.operands[1]);
                            let val = self.builder.reverse_bytes(src, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::REV16 => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let mut src = self.get_value(inst.operands[1]);
                            let mut res = self.builder.iconst(0);
                            let sixteen = self.builder.iconst(16);

                            let loop_iterations = match op_type {
                                I128 => 8,
                                I64 => 4,
                                _ => 2,
                            };
                            for _ in 0..loop_iterations {
                                let val = self.builder.reverse_bytes(src, I16);
                                res = self.builder.or(res, val, I16).into();
                                res = self.builder.ror(res, sixteen, op_type).into();
                                src = self.builder.ror(src, sixteen, op_type).into();
                            }
                            self.write_reg(res, dst_reg, op_type);
                        }
                        Opcode::REV32 => {
                            let dst_reg = self.get_dst_reg(inst);
                            let mut src = self.get_value(inst.operands[1]);
                            let mut res = self.builder.iconst(0);
                            let thirtytwo = self.builder.iconst(32);

                            let val = self.builder.reverse_bytes(src, I32);
                            res = self.builder.or(res, val, I32).into();
                            res = self.builder.ror(res, thirtytwo, I64).into();
                            src = self.builder.ror(src, thirtytwo, I64).into();

                            let val = self.builder.reverse_bytes(src, I32);
                            res = self.builder.or(res, val, I32).into();
                            res = self.builder.ror(res, thirtytwo, I64).into();

                            self.write_reg(res, dst_reg, I64);
                        }
                        Opcode::RORV => {
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let mask = match op_type {
                                I64 => self.builder.iconst(63),
                                _ => self.builder.iconst(31),
                            };
                            let shift = self.builder.and(src2, mask, op_type);
                            let val = self.builder.ror(src1, shift, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::SBC | Opcode::SBCS => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let carry = self.flag_value(Flag::C);
                            let carry = self.builder.bitwise_not(carry, BOOL);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.sub(src1, src2, op_type);
                            let val = self.builder.sub(val, carry, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            if inst.opcode == Opcode::SBCS {
                                let carry = self.flag_value(Flag::C);
                                self.set_flags_using_adc(src1, src2, op_type, carry);
                            }
                        }
                        Opcode::SBFM => {
                            let positive_condition_block =
                                self.builder.create_block("sbfm_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("sbfm_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = self.get_value(inst.operands[1]);
                            let immr = self.get_value(inst.operands[2]);
                            let imms = self.get_value(inst.operands[3]);
                            let cmp = self.builder.icmp(CmpTy::Uge, imms, immr, I64);
                            self.builder.jumpif(
                                cmp,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            let reg_size = match op_type {
                                I64 => self.builder.iconst(64),
                                _ => self.builder.iconst(32),
                            };

                            // copies a bitfield of (<imms>-<immr>+1) bits starting from bit position <immr> in the source register to the least significant bits of the destination register
                            self.builder.set_insert_block(positive_condition_block);
                            // get src bitfield
                            let one = self.builder.iconst(1);
                            let src_bitfield_size = self.builder.add(one, imms, op_type);
                            let src_bitfield_size =
                                self.builder.sub(src_bitfield_size, immr, op_type);
                            let shift_val = self.builder.add(imms, one, op_type);
                            let shift_val = self.builder.sub(reg_size, shift_val, op_type);
                            let val = self.builder.lshl(src, shift_val, op_type);
                            let shift_val = self.builder.sub(reg_size, src_bitfield_size, op_type);
                            let val = self.builder.ashr(val, shift_val, op_type);

                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            self.builder.set_insert_block(negative_condition_block);
                            let shift_val = self.builder.add(imms, one, op_type);
                            let shift_val = self.builder.sub(reg_size, shift_val, op_type);
                            let val = self.builder.lshl(src, shift_val, op_type);
                            let shift_val = self.builder.sub(reg_size, immr, op_type);
                            let val = self.builder.ashr(val, shift_val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::SDIV => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = self.builder.iconst(0);
                            let trap = self.builder.icmp(CmpTy::Eq, src2, zero, op_type);
                            self.builder.trapif(trap);
                            let val = self.builder.idiv(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::SMADDL => {
                            let dst_reg = self.get_dst_reg(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let src3 = self.get_value(inst.operands[3]);
                            let val = self.builder.imul(src1, src2, I32);
                            let val = self.builder.add(val, src3, I64);
                            self.write_reg(val, dst_reg, I64);
                        }
                        Opcode::SMC => {
                            // Ignoring secure monitor calls
                        }
                        Opcode::SMSUBL => {
                            let dst_reg = self.get_dst_reg(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let src3 = self.get_value(inst.operands[3]);
                            let val = self.builder.imul(src1, src2, I32);
                            let val = self.builder.sub(src3, val, I64);
                            self.write_reg(val, dst_reg, I64);
                        }
                        Opcode::SMULH => {
                            let dst_reg = self.get_dst_reg(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let val = self.builder.imul(src1, src2, I64);
                            let sixtyfour = self.builder.iconst(64);
                            let val = self.builder.ashr(val, sixtyfour, I128);
                            self.write_reg(val, dst_reg, I64);
                        }
                        Opcode::STP | Opcode::STNP => {
                            let src1 = self.get_value(inst.operands[0]);
                            let src2 = self.get_value(inst.operands[1]);
                            let address = self.get_value(inst.operands[2]);
                            let op_type = helper::get_type_by_inst(inst);

                            self.builder.store(src1, address, op_type);
                            let address_offset = match op_type {
                                I64 => self.builder.iconst(8),
                                _ => self.builder.iconst(4),
                            };
                            let address = self.builder.add(address, address_offset, I64);
                            self.builder.store(src2, address, op_type);
                        }
                        Opcode::STXP | Opcode::STLXP => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let address = self.get_value(inst.operands[3]);
                            let op_type = helper::get_type_by_inst(inst);

                            self.builder.store(src1, address, op_type);
                            let address_offset = match op_type {
                                I64 => self.builder.iconst(8),
                                _ => self.builder.iconst(4),
                            };
                            let address = self.builder.add(address, address_offset, I64);
                            self.builder.store(src2, address, op_type);
                            let dst_reg = self.get_dst_reg(inst);
                            let opaque = self.builder.opaque(op_type);
                            self.builder.write_reg(opaque, dst_reg, op_type);
                        }
                        Opcode::STR
                        | Opcode::STLR
                        | Opcode::STUR
                        | Opcode::STLUR
                        | Opcode::STTR => {
                            let op_type = helper::get_type_by_inst(inst);
                            let value = self.get_value(inst.operands[0]);
                            let address = self.get_value(inst.operands[1]);
                            self.builder.store(value, address, op_type);
                        }
                        Opcode::STLXR | Opcode::STXR => {
                            let op_type = helper::get_type_by_inst(inst);
                            let value = self.get_value(inst.operands[1]);
                            let address = self.get_value(inst.operands[2]);
                            self.builder.store(value, address, op_type);
                            let opaque = self.builder.opaque(op_type);
                            let dst_reg = self.get_dst_reg(inst);
                            self.write_reg(opaque, dst_reg, op_type);
                        }
                        Opcode::STRB
                        | Opcode::STLRB
                        | Opcode::STURB
                        | Opcode::STLURB
                        | Opcode::STTRB => {
                            let value = self.get_value(inst.operands[0]);
                            let address = self.get_value(inst.operands[1]);
                            self.builder.store(value, address, I8);
                        }
                        Opcode::STLXRB | Opcode::STXRB => {
                            let value = self.get_value(inst.operands[1]);
                            let address = self.get_value(inst.operands[2]);
                            self.builder.store(value, address, I8);
                            let dst_reg = self.get_dst_reg(inst);
                            let opaque = self.builder.opaque(I8);
                            self.write_reg(opaque, dst_reg, I8);
                        }
                        Opcode::STRH
                        | Opcode::STLRH
                        | Opcode::STURH
                        | Opcode::STLURH
                        | Opcode::STTRH => {
                            let value = self.get_value(inst.operands[0]);
                            let address = self.get_value(inst.operands[1]);
                            self.builder.store(value, address, I32);
                        }
                        Opcode::STLXRH | Opcode::STXRH => {
                            let value = self.get_value(inst.operands[1]);
                            let address = self.get_value(inst.operands[2]);
                            self.builder.store(value, address, I32);
                            let dst_reg = self.get_dst_reg(inst);
                            let opaque = self.builder.opaque(I32);
                            self.write_reg(opaque, dst_reg, I32);
                        }
                        Opcode::SUB | Opcode::SUBS => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let val = self.builder.sub(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            if inst.opcode == Opcode::SUBS {
                                let one = self.builder.iconst(1);
                                let not_src2 = self.builder.bitwise_not(src2, op_type).into();
                                self.set_flags_using_adc(src1, not_src2, op_type, one);
                            }
                        }
                        Opcode::SVC => {
                            // Ignoring supervisor calls
                            self.mark_next_block_as_entry(pc);
                        }
                        Opcode::SYS(_data) | Opcode::SYSL(_data) => {
                            // Ignoring system calls
                            self.mark_next_block_as_entry(pc);
                        }
                        Opcode::TBNZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let one = self.builder.iconst(1);
                            let zero = self.builder.iconst(0);
                            let src = self.get_reg_by_index(inst, 0);
                            let op_type = helper::get_type_by_inst(inst);
                            let test_bit = self.get_value(inst.operands[1]);
                            let offset = helper::get_pc_offset_as_int(inst.operands[2]);

                            let test_bit = self.builder.lshr(test_bit, one, op_type);
                            let val = self.builder.and(test_bit, src, op_type);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let jump_block = self.label_resolver.get_block(jump_address).unwrap();

                            let cmp = self.builder.icmp(CmpTy::Ne, val, zero, op_type);
                            self.builder.jumpif(
                                cmp,
                                jump_block,
                                Vec::new(),
                                next_block,
                                Vec::new(),
                            );
                        }
                        Opcode::TBZ => {
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let one = self.builder.iconst(1);
                            let zero = self.builder.iconst(0);
                            let src = self.get_reg_by_index(inst, 0);
                            let op_type = helper::get_type_by_inst(inst);
                            let test_bit = self.get_value(inst.operands[1]);
                            let offset = helper::get_pc_offset_as_int(inst.operands[2]);

                            let test_bit = self.builder.lshr(test_bit, one, op_type);
                            let val = self.builder.and(test_bit, src, op_type);
                            let jump_address = (pc as i64).wrapping_add(offset) as u64;
                            let jump_block = self.label_resolver.get_block(jump_address).unwrap();

                            let cmp = self.builder.icmp(CmpTy::Eq, val, zero, op_type);
                            self.builder.jumpif(
                                cmp,
                                jump_block,
                                Vec::new(),
                                next_block,
                                Vec::new(),
                            );
                        }
                        Opcode::UBFM => {
                            let positive_condition_block =
                                self.builder.create_block("ubfm_positive_condition", []);
                            let negative_condition_block =
                                self.builder.create_block("ubfm_negative_condition", []);
                            let next_address = pc + INSTRUCTION_SIZE;
                            let next_block = self.label_resolver.get_block(next_address).unwrap();

                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let src = self.get_value(inst.operands[1]);
                            let immr = self.get_value(inst.operands[2]);
                            let imms = self.get_value(inst.operands[3]);
                            let cmp = self.builder.icmp(CmpTy::Ult, immr, imms, I64);
                            self.builder.jumpif(
                                cmp,
                                positive_condition_block,
                                Vec::new(),
                                negative_condition_block,
                                Vec::new(),
                            );

                            let reg_size = match op_type {
                                I64 => self.builder.iconst(64),
                                _ => self.builder.iconst(32),
                            };

                            // copies a bitfield of (<imms>-<immr>+1) bits starting from bit position <immr> in the source register to the least significant bits of the destination register
                            self.builder.set_insert_block(positive_condition_block);
                            // get src bitfield
                            let one = self.builder.iconst(1);
                            let src_bitfield_size = self.builder.add(one, imms, op_type);
                            let src_bitfield_size =
                                self.builder.sub(src_bitfield_size, immr, op_type);
                            let shift_val = self.builder.add(imms, one, op_type);
                            let shift_val = self.builder.sub(reg_size, shift_val, op_type);
                            let val = self.builder.lshl(src, shift_val, op_type);
                            let shift_val = self.builder.sub(reg_size, src_bitfield_size, op_type);
                            let val = self.builder.lshr(val, shift_val, op_type);

                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());

                            // this copies a bitfield of (<imms>+1) bits from the least significant bits of the source register to bit position (regsize-<immr>) of the destination register
                            self.builder.set_insert_block(negative_condition_block);
                            let shift_val = self.builder.add(imms, one, op_type);
                            let shift_val = self.builder.sub(reg_size, shift_val, op_type);
                            let val = self.builder.lshl(src, shift_val, op_type);
                            let shift_val = self.builder.sub(reg_size, immr, op_type);
                            let val = self.builder.lshr(val, shift_val, op_type);
                            self.write_reg(val, dst_reg, op_type);
                            self.builder.jump(next_block, Vec::new());
                        }
                        Opcode::UDF => {
                            self.builder.trap();
                        }
                        Opcode::UDIV => {
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let dst_reg = self.get_dst_reg(inst);
                            let op_type = helper::get_type_by_inst(inst);
                            let zero = self.builder.iconst(0);
                            let trap = self.builder.icmp(CmpTy::Eq, src2, zero, op_type);
                            self.builder.trapif(trap);
                            let val = self.builder.udiv(src1, src2, op_type);
                            self.write_reg(val, dst_reg, op_type);
                        }
                        Opcode::UMADDL => {
                            let dst_reg = self.get_dst_reg(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let src3 = self.get_value(inst.operands[3]);
                            let val = self.builder.umul(src1, src2, I32);
                            let val = self.builder.add(val, src3, I64);
                            self.write_reg(val, dst_reg, I64);
                        }
                        Opcode::UMSUBL => {
                            let dst_reg = self.get_dst_reg(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let src3 = self.get_value(inst.operands[3]);
                            let val = self.builder.umul(src1, src2, I32);
                            let val = self.builder.sub(src3, val, I64);
                            self.write_reg(val, dst_reg, I64);
                        }
                        Opcode::UMULH => {
                            let dst_reg = self.get_dst_reg(inst);
                            let src1 = self.get_value(inst.operands[1]);
                            let src2 = self.get_value(inst.operands[2]);
                            let val = self.builder.umul(src1, src2, I64);
                            let sixtyfour = self.builder.iconst(64);
                            let val = self.builder.ashr(val, sixtyfour, I128);
                            self.write_reg(val, dst_reg, I64);
                        } // op => unimplemented!("{}", op),
                        _ => {
                            let is_general_purpose =
                                helper::is_operand_general_purpose(inst.operands[0]);
                            if is_general_purpose {
                                let dst_reg = self.get_dst_reg(inst);
                                let op_type = helper::get_type_by_inst(inst);
                                let val = self.builder.opaque(op_type);
                                self.write_reg(val, dst_reg, op_type);
                            }
                        }
                    }
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }

            pc += INSTRUCTION_SIZE;
        }

        Ok(())
    }

    /// Returns the value of a register as a 64-bit value.
    fn get_value(&mut self, operand: Operand) -> Value {
        match operand {
            Operand::Register(sz, reg) => self.reg_val(sz, reg, SpOrZrReg::Zr),
            Operand::RegisterOrSP(sz, reg) => self.reg_val(sz, reg, SpOrZrReg::Sp),
            Operand::Immediate(n) => self.builder.iconst(n),
            Operand::Imm64(n) => self.builder.iconst(n),
            Operand::Imm16(n) => self.builder.iconst(n),
            Operand::ImmShift(n, s) => self.builder.iconst((n as u64) << (s as u64)),
            Operand::ImmShiftMSL(n, s) => {
                let (n, s) = (n as u64, s as u64);
                let val = n << s;
                let mask = (1u64 << s) - 1;
                self.builder.iconst(val | mask)
            }
            Operand::RegShift(style, s, sz, reg) => {
                // 64 bit value, zero extended
                let reg_val = self.reg_val(sz, reg, SpOrZrReg::Zr);
                let op_type = helper::get_type_by_sizecode(sz);
                let shift_val = self.builder.iconst(s as u64);
                match style {
                    ShiftStyle::LSL | ShiftStyle::LSR if s == 0 => reg_val,
                    ShiftStyle::LSL => self.builder.lshl(reg_val, shift_val, op_type).into(),
                    ShiftStyle::LSR => self.builder.lshr(reg_val, shift_val, op_type).into(),
                    ShiftStyle::ASR => self.builder.ashr(reg_val, shift_val, op_type).into(),
                    ShiftStyle::ROR => self.builder.ror(reg_val, shift_val, op_type).into(),
                    ShiftStyle::UXTB | ShiftStyle::UXTH | ShiftStyle::UXTW | ShiftStyle::UXTX => {
                        reg_val
                    }
                    ShiftStyle::SXTB => {
                        // TODO: for this we might need some optimization later on.
                        let trunc = self.builder.trunc_i64(reg_val, I8);
                        self.builder.sext_i8(trunc, op_type).into()
                    }
                    ShiftStyle::SXTH => {
                        let trunc = self.builder.trunc_i64(reg_val, I16);
                        self.builder.sext_i16(trunc, op_type).into()
                    }
                    ShiftStyle::SXTW => {
                        let trunc = self.builder.trunc_i64(reg_val, I32);
                        self.builder.sext_i32(trunc, op_type).into()
                    }
                    ShiftStyle::SXTX => reg_val,
                }
            }
            Operand::RegRegOffset(rn, rd, sz, style, s) => {
                let rn = self.reg_val(SizeCode::X, rn, SpOrZrReg::Sp);
                let rd = self.reg_val(sz, rd, SpOrZrReg::Zr);
                let s = self.builder.iconst(if s == 1 { 2 } else { 0 });
                let op_type = helper::get_type_by_sizecode(sz);
                let offset_val = match style {
                    ShiftStyle::LSL => self.builder.lshl(rd, s, op_type).into(),
                    ShiftStyle::UXTW => rd,
                    ShiftStyle::SXTW => {
                        let trunc = self.builder.trunc_i64(rd, I32);
                        self.builder.sext_i32(trunc, I64).into()
                    }
                    ShiftStyle::SXTX => rd,
                    style => unimplemented!("RegRegOffset with style: {:?}", style),
                };
                self.builder.add(rn, offset_val, I64).into()
            }
            Operand::RegPreIndex(rn, offset, _write_back) => {
                let rn = self.reg_val(SizeCode::X, rn, SpOrZrReg::Sp);
                let offset = self.builder.iconst(offset as u64);
                self.builder.add(rn, offset, I64).into()
            }
            Operand::RegPostIndex(rn, offset) => {
                let val = self.reg_val(SizeCode::X, rn, SpOrZrReg::Sp);
                let offset = self.builder.iconst(offset as u64);
                self.builder.add(val, offset, I64).into()
            }
            Operand::RegPostIndexReg(_, _) => unimplemented!("RegPostIndexReg"),
            Operand::PCOffset(n) => self.builder.iconst(n as u64),
            _ => self
                .builder
                .opaque(helper::get_type_by_operand(operand))
                .into(),
        }
    }

    /// reads a register value
    fn reg_val(&mut self, sz: SizeCode, reg: u16, sp_or_zr: SpOrZrReg) -> Value {
        let op_type = helper::get_type_by_sizecode(sz);
        let val = if reg == 31 {
            match sp_or_zr {
                SpOrZrReg::Sp => self.builder.read_reg(
                    self.builder
                        .get_code_region()
                        .get_arch()
                        .lookup_reg(&"sp".into())
                        .unwrap(),
                    I64,
                ),
                SpOrZrReg::Zr => {
                    return self.builder.opaque(op_type).into();
                }
            }
        } else {
            self.builder.read_reg(Reg(reg as u32), op_type)
        };
        val.into()
    }

    fn get_reg_by_index(&self, inst: Instruction, index: usize) -> Reg {
        let dst_reg = match inst.operands[index] {
            Operand::Register(_, reg) => Reg(reg as u32),
            Operand::RegisterOrSP(sz, reg) => {
                if reg == 31 {
                    assert_eq!(sz, SizeCode::X, "sp must be 64 bits");
                    self.builder
                        .get_code_region()
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

    fn get_dst_reg(&self, inst: Instruction) -> Reg {
        self.get_reg_by_index(inst, 0)
    }

    fn flag_value(&mut self, flag: Flag) -> Value {
        let reg = match flag {
            Flag::N => "n",
            Flag::Z => "z",
            Flag::C => "c",
            Flag::V => "v",
        };
        self.builder
            .read_reg(
                self.builder
                    .get_code_region()
                    .get_arch()
                    .lookup_reg(&reg.into())
                    .unwrap(),
                BOOL,
            )
            .into()
    }

    fn write_flag(&mut self, value: Value, flag: Flag) {
        let reg_name = match flag {
            Flag::N => "n",
            Flag::Z => "z",
            Flag::C => "c",
            Flag::V => "v",
        };
        let reg = self.get_reg_val_by_name(reg_name);
        self.write_reg(value, reg, BOOL);
    }

    fn get_reg_val_by_name(&mut self, name: &str) -> Reg {
        self.builder
            .get_code_region()
            .get_arch()
            .lookup_reg(&name.into())
            .unwrap()
    }

    fn read_pc_reg(&mut self) -> Value {
        let reg = self.get_reg_val_by_name("pc");
        self.reg_val(SizeCode::X, reg.0 as u16, SpOrZrReg::Sp)
    }

    fn get_condition(&mut self, operand: Operand) -> Result<Inst, AArch64LifterError> {
        let one = self.builder.iconst(1);
        match operand {
            Operand::ConditionCode(cc) => {
                let inst = match cc {
                    0 => {
                        // EQ
                        let z = self.flag_value(Flag::Z);
                        self.builder.icmp(CmpTy::Eq, z, one, BOOL)
                    }
                    1 => {
                        // NE
                        let z = self.flag_value(Flag::Z);
                        self.builder.icmp(CmpTy::Ne, z, one, BOOL)
                    }
                    2 => {
                        // CS
                        let c = self.flag_value(Flag::C);
                        self.builder.icmp(CmpTy::Eq, c, one, BOOL)
                    }
                    3 => {
                        // CC
                        let c = self.flag_value(Flag::C);
                        self.builder.icmp(CmpTy::Ne, c, one, BOOL)
                    }
                    4 => {
                        // MI
                        let n = self.flag_value(Flag::N);
                        self.builder.icmp(CmpTy::Eq, n, one, BOOL)
                    }
                    5 => {
                        // PL
                        let n = self.flag_value(Flag::N);
                        self.builder.icmp(CmpTy::Ne, n, one, BOOL)
                    }
                    6 => {
                        // VS
                        let v = self.flag_value(Flag::V);
                        self.builder.icmp(CmpTy::Eq, v, one, BOOL)
                    }
                    7 => {
                        // VC
                        let v = self.flag_value(Flag::V);
                        self.builder.icmp(CmpTy::Ne, v, one, BOOL)
                    }
                    8 => {
                        // HI
                        let z = self.flag_value(Flag::Z);
                        let c = self.flag_value(Flag::C);

                        let c_is_true = self.builder.icmp(CmpTy::Eq, c, one, BOOL);
                        let z_is_false = self.builder.icmp(CmpTy::Ne, z, one, BOOL);
                        self.builder.and(c_is_true, z_is_false, BOOL)
                    }
                    9 => {
                        // LS
                        let z = self.flag_value(Flag::Z);
                        let c = self.flag_value(Flag::C);

                        let c_is_false: Inst = self.builder.icmp(CmpTy::Ne, c, one, BOOL);

                        let z_is_true = self.builder.icmp(CmpTy::Eq, z, one, BOOL);
                        self.builder.or(c_is_false, z_is_true, BOOL)
                    }
                    10 => {
                        // GE
                        let n = self.flag_value(Flag::N);
                        let v = self.flag_value(Flag::V);

                        self.builder.icmp(CmpTy::Eq, n, v, BOOL)
                    }
                    11 => {
                        // LT
                        let n = self.flag_value(Flag::N);
                        let v = self.flag_value(Flag::V);

                        self.builder.icmp(CmpTy::Ne, n, v, BOOL)
                    }
                    12 => {
                        // GT
                        let z = self.flag_value(Flag::Z);
                        let n = self.flag_value(Flag::N);
                        let v = self.flag_value(Flag::V);

                        let z_is_false = self.builder.icmp(CmpTy::Ne, z, one, BOOL);
                        let n_eq_v = self.builder.icmp(CmpTy::Eq, n, v, BOOL);
                        self.builder.and(z_is_false, n_eq_v, BOOL)
                    }
                    13 => {
                        // LE
                        let z = self.flag_value(Flag::Z);
                        let n = self.flag_value(Flag::N);
                        let v = self.flag_value(Flag::V);

                        let z_is_true = self.builder.icmp(CmpTy::Eq, z, one, BOOL);
                        let n_neq_v = self.builder.icmp(CmpTy::Ne, n, v, BOOL);
                        self.builder.or(z_is_true, n_neq_v, BOOL)
                    }
                    14 => {
                        // AL
                        self.builder.and(one, one, BOOL)
                    }
                    15 => {
                        // NV
                        self.builder.icmp(CmpTy::Ne, one, one, BOOL)
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

    fn set_flags_to_value(&mut self, flag_val: Value, op_type: Type) {
        let zero = self.builder.iconst(0);
        // set n flag
        let n_mask = self.builder.iconst(8);
        let n = self.builder.and(n_mask, flag_val, op_type);
        let n_is_set = self.builder.icmp(CmpTy::Ne, zero, n, op_type);
        self.write_flag(n_is_set.into(), Flag::N);
        // set z flag
        let z_mask = self.builder.iconst(4);
        let z = self.builder.and(z_mask, flag_val, op_type);
        let z_is_set = self.builder.icmp(CmpTy::Ne, zero, z, op_type);
        self.write_flag(z_is_set.into(), Flag::Z);
        // set c flag
        let c_mask = self.builder.iconst(2);
        let c = self.builder.and(c_mask, flag_val, op_type);
        let c_is_set = self.builder.icmp(CmpTy::Ne, zero, c, op_type);
        self.write_flag(c_is_set.into(), Flag::C);
        // set v flag
        let v_mask = self.builder.iconst(1);
        let v = self.builder.and(v_mask, flag_val, op_type);
        let v_is_set = self.builder.icmp(CmpTy::Ne, zero, v, op_type);
        self.write_flag(v_is_set.into(), Flag::V);
    }

    fn set_flags_using_adc(&mut self, val1: Value, val2: Value, op_type: Type, carry: Value) {
        let zero = self.builder.iconst(0);
        let sum = self.builder.add(val1, val2, op_type);
        let sum = self.builder.add(sum, carry, op_type);

        // z is set if equal if both values are equal
        let z = self.builder.icmp(CmpTy::Eq, sum, zero, op_type);
        self.write_flag(z.into(), Flag::Z);
        // n is set if the sum is negative
        let n = self.builder.icmp(CmpTy::Slt, sum, zero, op_type);
        self.write_flag(n.into(), Flag::N);
        // if either operand is greater than the result in an unsigned comparison, the carry is set
        let val1_is_ugt_sum = self.builder.icmp(CmpTy::Ugt, val1, sum, op_type);
        let val2_is_ugt_sum = self.builder.icmp(CmpTy::Ugt, val2, sum, op_type);
        let c = self.builder.or(val1_is_ugt_sum, val2_is_ugt_sum, BOOL);
        self.write_flag(c.into(), Flag::C);
        // v is set if both operands have the same sign and the result has a different sign
        let val1_is_negative = self.builder.icmp(CmpTy::Slt, val1, zero, op_type);
        let val2_is_negative = self.builder.icmp(CmpTy::Slt, val2, zero, op_type);
        let values_have_same_sign =
            self.builder
                .icmp(CmpTy::Eq, val1_is_negative, val2_is_negative, BOOL);
        let result_has_different_sign = self.builder.icmp(CmpTy::Ne, val1_is_negative, n, BOOL);
        let v = self
            .builder
            .and(values_have_same_sign, result_has_different_sign, BOOL);
        self.write_flag(v.into(), Flag::V);
    }

    fn write_reg(&mut self, val: impl Into<Value>, dst_reg: Reg, op_type: Type) {
        if dst_reg.0 != 31 {
            self.builder.write_reg(val, dst_reg, op_type);
        }
    }

    fn mark_next_block_as_entry(&mut self, pc: u64) {
        let next_pc = pc + INSTRUCTION_SIZE;
        let block = self
            .label_resolver
            .get_block(next_pc)
            .expect("next block to exist");
        self.builder.mark_entry_block(block);
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
