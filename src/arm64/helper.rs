use tnj::types::{Type, I32, I64};
use yaxpeax_arm::armv8::a64::{Operand, SizeCode};

pub fn get_pc_offset_as_int(operand: Operand) -> u64 {
    match operand {
        Operand::PCOffset(imm) => imm as u64,
        op => unimplemented!("dst op {:?}", op),
    }
}

pub fn get_block_name(jump_address: u64) -> String {
    format!("block_{}", jump_address)
}

pub fn get_type_by_sizecode(sz: SizeCode) -> Type {
    match sz {
        SizeCode::X => I64,
        SizeCode::W => I32,
    }
}
