use crate::arm64::lifter::{Flag, LifterState, INSTRUCTION_SIZE};
use crate::arm64::{helper, AArch64LifterError};
use tnj::types::cmp::CmpTy;
use tnj::types::{BOOL, I128, I16, I32, I64, I8};
use yaxpeax_arm::armv8::a64::{Instruction, Opcode, Operand};

impl LifterState<'_> {
    pub(crate) fn lift_inst(
        &mut self,
        pc: u64,
        inst: Instruction,
    ) -> Result<(), AArch64LifterError> {
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
                    let return_address = self.builder.add(pc_reg, instruction_size, I64);
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
                self.builder
                    .jumpif(condition, jump_block, Vec::new(), next_block, Vec::new());
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
                let src_bitfield_size = self.builder.sub(src_bitfield_size, immr, op_type);
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
                let src_bitfield = self.builder.lshl(src_bitfield, starting_position, op_type);
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
                self.builder
                    .jumpif(cmp, swap_block, Vec::new(), next_block, Vec::new());

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

                self.builder
                    .jumpif(condition, block, Vec::new(), next_block, Vec::new());
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

                self.builder
                    .jumpif(condition, block, Vec::new(), next_block, Vec::new());
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
                let target = self.get_value(inst.operands[0]);
                self.builder.dynamic_jump(target);
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
                let src_bitfield_size = self.builder.sub(src_bitfield_size, immr, op_type);
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
            Opcode::STR | Opcode::STLR | Opcode::STUR | Opcode::STLUR | Opcode::STTR => {
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
            Opcode::STRB | Opcode::STLRB | Opcode::STURB | Opcode::STLURB | Opcode::STTRB => {
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
            Opcode::STRH | Opcode::STLRH | Opcode::STURH | Opcode::STLURH | Opcode::STTRH => {
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
                self.builder
                    .jumpif(cmp, jump_block, Vec::new(), next_block, Vec::new());
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
                self.builder
                    .jumpif(cmp, jump_block, Vec::new(), next_block, Vec::new());
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
                let src_bitfield_size = self.builder.sub(src_bitfield_size, immr, op_type);
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
                let is_general_purpose = helper::is_operand_general_purpose(inst.operands[0]);
                if is_general_purpose {
                    let dst_reg = self.get_dst_reg(inst);
                    let op_type = helper::get_type_by_inst(inst);
                    let val = self.builder.opaque(op_type);
                    self.write_reg(val, dst_reg, op_type);
                }
            }
        }
        Ok(())
    }
}
