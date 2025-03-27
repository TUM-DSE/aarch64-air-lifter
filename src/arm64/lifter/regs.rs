use crate::arm64::helper;
use crate::arm64::lifter::{LifterState, SpOrZrReg};
use tnj::air::instructions::Value;
use tnj::arch::reg::Reg;
use tnj::types::{Type, I64};
use yaxpeax_arm::armv8::a64::{Instruction, Operand, SizeCode};

impl LifterState<'_> {
    /// reads a register value
    pub(crate) fn reg_val(&mut self, sz: SizeCode, reg: u16, sp_or_zr: SpOrZrReg) -> Value {
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

    pub(crate) fn get_reg_by_index(&self, inst: Instruction, index: usize) -> Option<Reg> {
        let dst_reg = match inst.operands[index] {
            Operand::Register(_, reg) => Some(Reg(reg as u32)),
            Operand::RegisterOrSP(sz, reg) => {
                if reg == 31 {
                    assert_eq!(sz, SizeCode::X, "sp must be 64 bits");
                    Some(
                        self.builder
                            .get_code_region()
                            .get_arch()
                            .lookup_reg(&"sp".into())
                            .unwrap(),
                    )
                } else {
                    Some(Reg(reg as u32))
                }
            }
            _ => None,
        };
        dst_reg
    }

    pub(crate) fn get_dst_reg(&self, inst: Instruction) -> Option<Reg> {
        self.get_reg_by_index(inst, 0)
    }

    pub(crate) fn read_pc_reg(&mut self) -> Value {
        let reg = self.get_reg_val_by_name("pc");
        self.reg_val(SizeCode::X, reg.0 as u16, SpOrZrReg::Sp)
    }

    pub(crate) fn get_reg_val_by_name(&mut self, name: &str) -> Reg {
        self.builder
            .get_code_region()
            .get_arch()
            .lookup_reg(&name.into())
            .unwrap()
    }

    pub(crate) fn write_reg(&mut self, val: impl Into<Value>, dst_reg: Reg, op_type: Type) {
        assert_ne!(dst_reg.0, 31, "cannot write to register 31");
        self.builder.write_reg(val, dst_reg, op_type);
    }
}
