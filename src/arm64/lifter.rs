#![allow(warnings)]
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
        let mut label_resolver = label_resolver::LabelResolver::new(code, &mut builder, &decoder)?;

        let mut address: isize = 0;

        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    println!("{}", inst);
                    match inst.opcode {
                        // Currently not supported
                        Opcode::ADD => {
                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let val = builder.add(src1, src2, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::ADC => {
                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let carry = Self::flag_value(&mut builder, Flag::C);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let val = builder.add(src1, carry, I64);
                            let val = builder.add(val, src2, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::AND => {
                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let val = builder.and(src1, src2, I32);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::B => {
                            let offset = helper::get_pc_offset(inst.operands[0]);
                            let jump_address = address as isize + offset;
                            let block_name = helper::get_block_name(jump_address);
                            let block = label_resolver.get_block(&block_name);
                            let block = match block {
                                Some(block) => block,
                                None => {
                                    return Err(AArch64LifterError::CustomError(
                                        "Jumping to resolved block that does not exist".to_string(),
                                    ));
                                }
                            };
                            builder.jump(*block, vec![]);
                            builder.set_insert_block(*block);
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
                            let current_block = builder.current_block();
                            let end_block =
                                builder.create_block("csinc_end", Vec::<BlockParamData>::new());

                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
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
                            let val = builder.add(src2, one, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                            builder.jump(end_block, Vec::new());

                            // Condition is true
                            builder.set_insert_block(positive_condition_block);
                            if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(src1, I32);
                                let val = builder.zext_i32(trunc, I64);
                                builder.write_reg(val, dst_reg, I64);
                            } else {
                                builder.write_reg(src1, dst_reg, I64);
                            };
                            builder.jump(end_block, Vec::new());

                            builder.set_insert_block(end_block);
                        }
                        Opcode::NEG => {
                            let zero = builder.iconst(0);
                            let src = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let val = builder.sub(zero, src, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::ORR => {
                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let val = builder.or(src1, src2, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::ORN => {
                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let val = builder.not(src2, I64);
                            let val = builder.or(src1, val, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                        }
                        Opcode::SUB => {
                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let val = builder.sub(src1, src2, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                        }
                        op => unimplemented!("{}", op),
                    }
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }

            address += INSTRUCTION_SIZE;
        }

        Ok(blob)
    }
}

impl AArch64Lifter {
    /// Returns the value of a register as a 64-bit value.
    fn get_reg_value(builder: &mut InstructionBuilder, operand: Operand) -> Value {
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
                let shift_val = builder.iconst(s as u64);
                match style {
                    ShiftStyle::LSL | ShiftStyle::LSR if s == 0 => reg_val,
                    ShiftStyle::LSL => builder.lshl(reg_val, shift_val, I64).into(),
                    ShiftStyle::LSR => builder.lshr(reg_val, shift_val, I64).into(),
                    ShiftStyle::ASR => unimplemented!("ASR"),
                    ShiftStyle::ROR => unimplemented!("ROR"),
                    ShiftStyle::UXTB | ShiftStyle::UXTH | ShiftStyle::UXTW | ShiftStyle::UXTX => {
                        reg_val
                    }
                    ShiftStyle::SXTB => {
                        // TODO: for this we might need some optimization later on.
                        let trunc = builder.trunc_i64(reg_val, I8);
                        builder.sext_i8(trunc, I64).into()
                    }
                    ShiftStyle::SXTH => {
                        let trunc = builder.trunc_i64(reg_val, I16);
                        builder.sext_i16(trunc, I64).into()
                    }
                    ShiftStyle::SXTW => {
                        let trunc = builder.trunc_i64(reg_val, I32);
                        builder.sext_i32(trunc, I64).into()
                    }
                    ShiftStyle::SXTX => reg_val,
                }
            }
            Operand::RegRegOffset(rn, rd, sz, style, s) => {
                let rn = Self::reg_val(builder, sz, rn, SpOrZrReg::Sp);
                let rd = Self::reg_val(builder, sz, rd, SpOrZrReg::Zr);
                let s = builder.iconst(if s == 1 { 2 } else { 0 });
                let offset_val = match style {
                    ShiftStyle::LSL => builder.lshl(rd, s, I64).into(),
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
            Operand::RegPostIndex(rn, _) => Self::reg_val(builder, SizeCode::X, rn, SpOrZrReg::Sp),
            Operand::RegPostIndexReg(_, _) => unimplemented!("RegPostIndexReg"),
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
            builder.read_reg(Reg(reg as u32), I64)
        };
        if sz == SizeCode::W {
            // then we need to trunc the top bits
            let trunc = builder.trunc_i64(val, I32);
            builder.zext_i32(trunc, I64)
        } else {
            val
        }
        .into()
    }

    fn get_dst_reg(builder: &InstructionBuilder, inst: Instruction) -> (Reg, SizeCode) {
        let (dst_reg, sz) = match inst.operands[0] {
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
                        // TODO: Change to prettier code?
                        // AL
                        builder.and(flag_is_true, flag_is_true, BOOL)
                    }
                    15 => {
                        // NV
                        builder.and(flag_is_true, flag_is_false, BOOL)
                    }
                    _ => {
                        return Err(AArch64LifterError::CustomError(
                            "Invalid condition code".to_string(),
                        ));
                    }
                };
                Ok(inst)
            }
            _ => {
                return Err(AArch64LifterError::CustomError(
                    "Invalid operand for condition code".to_string(),
                ));
            }
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
