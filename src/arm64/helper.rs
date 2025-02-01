use tnj::types::{Type, I128, I16, I32, I64, I8};
use yaxpeax_arm::armv8::a64::Instruction;
use yaxpeax_arm::armv8::a64::{Operand, SIMDSizeCode, SizeCode};

pub fn get_pc_offset_as_int(operand: Operand) -> i64 {
    match operand {
        Operand::PCOffset(imm) => imm,
        op => unimplemented!("dst op {:?}", op),
    }
}

pub fn get_block_name(jump_address: u64) -> String {
    format!("block_{}", jump_address)
}

pub fn get_type_by_inst(inst: Instruction) -> Type {
    match inst.operands[0] {
        Operand::Register(sz, _) => get_type_by_sizecode(sz),
        Operand::RegisterOrSP(sz, _) => get_type_by_sizecode(sz),
        Operand::RegisterPair(sz, _) => get_type_by_sizecode(sz),
        Operand::SIMDRegister(sz, _) => get_type_by_simd_sizecode(sz),
        Operand::SIMDRegisterElements(sz, _, _) => get_type_by_simd_sizecode(sz),
        Operand::SIMDRegisterElementsLane(_, _, _, _) => {
            unimplemented!("TODO: Implement get_type_by_inst for SIMDRegisterElementsLane")
        }
        Operand::SIMDRegisterElementsMultipleLane(_, _, _, _, _) => {
            unimplemented!("TODO: Implement get_type_by_inst for SIMDRegisterElementsMultipleLane")
        }
        Operand::SIMDRegisterGroup(_, _, _, _) => {
            unimplemented!("TODO: Implement get_type_by_inst for SIMDRegisterGroup")
        }
        Operand::SIMDRegisterGroupLane(_, _, _, _) => {
            unimplemented!("TODO: Implement get_type_by_inst for SIMDRegisterGroupLane")
        }
        op => unimplemented!("Destination operand invalid {:?}", op),
    }
}

pub fn get_type_by_sizecode(sz: SizeCode) -> Type {
    match sz {
        SizeCode::W => I32,
        SizeCode::X => I64,
    }
}

pub fn get_type_by_simd_sizecode(sz: SIMDSizeCode) -> Type {
    match sz {
        SIMDSizeCode::B => I8,
        SIMDSizeCode::H => I16,
        SIMDSizeCode::S => I32,
        SIMDSizeCode::D => I64,
        SIMDSizeCode::Q => I128,
    }
}
