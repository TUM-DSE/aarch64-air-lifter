use crate::arm64::helper;
use crate::arm64::lifter::{LifterState, SpOrZrReg};
use tnj::air::instructions::Value;
use tnj::types::{I16, I32, I64, I8};
use yaxpeax_arm::armv8::a64::{Operand, ShiftStyle, SizeCode};

impl LifterState<'_> {
    /// Returns the value of a register as a 64-bit value.
    pub(crate) fn get_value(&mut self, operand: Operand) -> Value {
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
                        if op_type.bit_width().expect("type to be bit vector") > 32 {
                            self.builder.sext_i32(trunc, op_type).into()
                        } else {
                            trunc.into()
                        }
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
                self.builder.wrapping_add(rn, offset_val, I64).into()
            }
            Operand::RegPreIndex(rn, offset, _write_back) => {
                let rn = self.reg_val(SizeCode::X, rn, SpOrZrReg::Sp);
                let offset = self.builder.iconst(offset as u64);
                self.builder.wrapping_add(rn, offset, I64).into()
            }
            Operand::RegPostIndex(rn, offset) => {
                let val = self.reg_val(SizeCode::X, rn, SpOrZrReg::Sp);
                let offset = self.builder.iconst(offset as u64);
                self.builder.wrapping_add(val, offset, I64).into()
            }
            Operand::RegPostIndexReg(_, _) => unimplemented!("RegPostIndexReg"),
            Operand::PCOffset(n) => self.builder.iconst(n as u64),
            _ => self
                .builder
                .opaque(helper::get_type_by_operand(operand))
                .into(),
        }
    }

    pub fn is_simd_register(operand: Operand) -> bool {
        matches!(
            operand,
            Operand::SIMDRegister(..)
                | Operand::SIMDRegisterElements(..)
                | Operand::SIMDRegisterElementsLane(..)
                | Operand::SIMDRegisterElementsMultipleLane(..)
                | Operand::SIMDRegisterGroup(..)
                | Operand::SIMDRegisterGroupLane(..)
        )
    }
}
