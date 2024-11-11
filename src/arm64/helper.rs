use yaxpeax_arm::armv8::a64::Operand;

pub fn get_pc_offset(operand: Operand) -> isize {
    match operand {
        Operand::PCOffset(imm) => imm as isize,
        op => unimplemented!("dst op {:?}", op),
    }
}

pub fn get_block_name(jump_address: isize) -> String {
    format!("block_{}", jump_address)
}
