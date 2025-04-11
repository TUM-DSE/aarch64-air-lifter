use crate::arm64::lifter::{Flag, LifterState};
use crate::arm64::AArch64LifterError;
use air::instructions::Inst;
use types::cmp::CmpTy;
use types::BOOL;
use yaxpeax_arm::armv8::a64::Operand;

impl LifterState<'_> {
    pub(crate) fn get_condition(&mut self, operand: Operand) -> Result<Inst, AArch64LifterError> {
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
}
