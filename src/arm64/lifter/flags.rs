use crate::arm64::lifter::{Flag, LifterState};
use air::instructions::Value;
use types::cmp::CmpTy;
use types::{Type, BOOL};

impl LifterState<'_> {
    pub(crate) fn flag_value(&mut self, flag: Flag) -> Value {
        let reg = get_flag_name(flag);
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

    pub(crate) fn write_flag(&mut self, value: Value, flag: Flag) {
        let reg_name = get_flag_name(flag);
        let reg = self.get_reg_val_by_name(reg_name);
        self.write_reg(value, reg, BOOL);
    }

    pub(crate) fn set_flags_to_value(&mut self, flag_val: Value, op_type: Type) {
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

    pub(crate) fn set_flags_using_adc(
        &mut self,
        val1: Value,
        val2: Value,
        op_type: Type,
        carry: Value,
    ) {
        let zero = self.builder.iconst(0);
        let sum = self.builder.wrapping_add(val1, val2, op_type);
        let sum = self.builder.wrapping_add(sum, carry, op_type);

        // z is set if equal if both values are equal
        let z = self.builder.icmp(CmpTy::Eq, sum, zero, op_type);
        self.write_flag(z.into(), Flag::Z);
        // n is set if the sum is negative
        let n = self.builder.scmp(CmpTy::Lt, sum, zero, op_type);
        self.write_flag(n.into(), Flag::N);
        // if either operand is greater than the result in an unsigned comparison, the carry is set
        let val1_is_ugt_sum = self.builder.ucmp(CmpTy::Gt, val1, sum, op_type);
        let val2_is_ugt_sum = self.builder.ucmp(CmpTy::Gt, val2, sum, op_type);
        let c = self.builder.or(val1_is_ugt_sum, val2_is_ugt_sum, BOOL);
        self.write_flag(c.into(), Flag::C);
        // v is set if both operands have the same sign and the result has a different sign
        let val1_is_negative = self.builder.scmp(CmpTy::Lt, val1, zero, op_type);
        let val2_is_negative = self.builder.scmp(CmpTy::Lt, val2, zero, op_type);
        let values_have_same_sign =
            self.builder
                .icmp(CmpTy::Eq, val1_is_negative, val2_is_negative, BOOL);
        let result_has_different_sign = self.builder.icmp(CmpTy::Ne, val1_is_negative, n, BOOL);
        let v = self
            .builder
            .and(values_have_same_sign, result_has_different_sign, BOOL);
        self.write_flag(v.into(), Flag::V);
    }
}

fn get_flag_name(flag: Flag) -> &'static str {
    match flag {
        Flag::N => "n",
        Flag::Z => "z",
        Flag::C => "c",
        Flag::V => "v",
    }
}
