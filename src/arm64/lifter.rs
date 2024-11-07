use crate::Lifter;
use target_lexicon::{Aarch64Architecture, Architecture};
use thiserror::Error;
use tnj::air::instructions::builder::InstructionBuilder;
use tnj::air::instructions::{Blob, BlockParamData, CmpTy, Value};
use tnj::arch::get_arch;
use tnj::arch::reg::Reg;
use tnj::types::{I16, I32, I64, I8};
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{
    ARMv8, DecodeError, Instruction, Opcode, Operand, ShiftStyle, SizeCode,
};

/// A lifter for AArch64
pub struct AArch64Lifter;

impl Lifter for AArch64Lifter {
    type E = AArch64LifterError;

    fn lift(&self, code: &[u8], _proofs: &[u8]) -> Result<Blob, Self::E> {
        let arch = get_arch(Architecture::Aarch64(Aarch64Architecture::Aarch64)).unwrap();
        let mut blob = Blob::new(arch);
        let mut builder = blob.insert();

        let decoder = <ARMv8 as Arch>::Decoder::default();

        let mut reader = U8Reader::new(code);

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
                        Opcode::B => {
                            // let target = Self::get_reg_value(&mut builder, inst.operands[1]);
                            // builder.jump(dest, dest_params)
                        }
                        Opcode::CSINC => {
                            let final_block =
                                builder.create_block("Final Block", Vec::<BlockParamData>::new());
                            let conditional_block = builder
                                .create_block("Conditional Block", Vec::<BlockParamData>::new());
                            let current_block = builder.current_block();

                            let src1 = Self::get_reg_value(&mut builder, inst.operands[1]);
                            let src2 = Self::get_reg_value(&mut builder, inst.operands[2]);
                            let (dst_reg, sz) = Self::get_dst_reg(&builder, inst);
                            let cmp_ty = Self::get_condition_code(inst.operands[3]);
                            let cmp = builder.icmp(cmp_ty, src1, src2, I64);
                            builder.jumpif(
                                cmp,
                                conditional_block,
                                Vec::new(),
                                current_block,
                                Vec::new(),
                            );

                            // Condition is false
                            let one = builder.iconst(1);
                            let val = builder.add(src2, one, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                            builder.jump(final_block, Vec::new());

                            // Condition is true
                            builder.set_insert_block(conditional_block);
                            let zero = builder.iconst(0);
                            let val = builder.add(src1, zero, I64);
                            let val = if sz == SizeCode::W {
                                let trunc = builder.trunc_i64(val, I32);
                                builder.zext_i32(trunc, I64)
                            } else {
                                val
                            };
                            builder.write_reg(val, dst_reg, I64);
                            builder.jump(final_block, Vec::new());

                            builder.set_insert_block(final_block);
                        }
                        op => unimplemented!("{}", op),
                    }
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }
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

    fn get_condition_code(operand: Operand) -> CmpTy {
        match operand {
            Operand::ConditionCode(c) => match c {
                0 => CmpTy::Eq,
                1 => CmpTy::Ne,
                2 => CmpTy::Ugt,
                3 => CmpTy::Uge,
                4 => CmpTy::Ult,
                5 => CmpTy::Ule,
                6 => CmpTy::Sgt,
                7 => CmpTy::Sge,
                8 => CmpTy::Slt,
                9 => CmpTy::Sle,
                10..=15 => unimplemented!("condition code: {}", c),
                _ => unreachable!("incorrect condition code: {}", c),
            },
            _ => unreachable!("incorrect operand for `get_condition_code`: {:?}", operand),
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
}
