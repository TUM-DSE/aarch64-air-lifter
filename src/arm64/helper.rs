use tnj::air::instructions::CmpTy;
use yaxpeax_arm::armv8::a64::Operand;

pub fn get_pc_offset(operand: Operand) -> isize {
    match operand {
        Operand::PCOffset(imm) => imm as isize,
        op => unimplemented!("dst op {:?}", op),
    }
}

pub fn get_condition_code(operand: Operand) -> CmpTy {
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

pub fn get_block_name(jump_address: isize) -> String {
    format!("block_{:x}", jump_address)
}
