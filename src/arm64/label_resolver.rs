#![allow(warnings)]
#![allow(dead_code)]
#![allow(unused_imports)]

use crate::Lifter;
use std::{collections::BinaryHeap, io::Error};
use target_lexicon::Aarch64Architecture;
use tnj::air::instructions::{builder::InstructionBuilder, Inst};
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{
    ARMv8, DecodeError, InstDecoder, Instruction, Opcode, Operand, ShiftStyle, SizeCode,
};

use super::AArch64LifterError;

/// Create basic blocks for the InstructionBuilder based off labels
pub struct LabelResolver {
    checkpoints: BinaryHeap<isize>,
}

impl LabelResolver {
    fn new() -> Self {
        Self {
            checkpoints: BinaryHeap::new(),
        }
    }

    /*
        High-Level:
        Goes through all instructions and creates checkpoints
        A checkpoint is where a label is jumping to or where a label is

        After gathering all the checkpoints, it creates the blocks
    */

    fn _resolve(&mut self, code: &[u8], builder: &mut InstructionBuilder, decoder: &InstDecoder) {
        self.get_checkpoints(code, decoder);
        self.create_blocks(builder);
    }

    /*
        Wir brauchen einen Basic block:
        1. wenn vorher ein jump stattgefunden hat
        2. wenn wir zu dieser instruktion springen können
    */
    fn get_checkpoints(
        &mut self,
        code: &[u8],
        decoder: &InstDecoder,
    ) -> Result<(), AArch64LifterError> {
        let mut reader = U8Reader::new(code);
        let mut address: isize = -1;
        loop {
            address += 1;
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    println!("{}", inst);
                    let imm: isize = match inst.opcode {
                        Opcode::B | Opcode::BL | Opcode::Bcc(_) => {
                            self.get_pc_offset(inst.operands[0])
                        }
                        Opcode::CBNZ | Opcode::CBZ | Opcode::TBL | Opcode::TBX => {
                            self.get_pc_offset(inst.operands[1])
                        }
                        Opcode::TBNZ | Opcode::TBZ => self.get_pc_offset(inst.operands[2]),
                        Opcode::BLR | Opcode::BR => {
                            // TODO: Uses dynamic address. Might need to be handled in the future differently
                            continue;
                        }
                        _ => continue,
                    };
                    self.checkpoints.push(address + 1);
                    let jump_address = imm + address;
                    self.checkpoints.push(jump_address);
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }
            address += 1;
        }
        Ok(())
    }

    fn get_pc_offset(&self, operand: Operand) -> isize {
        match operand {
            Operand::PCOffset(imm) => imm as isize,
            op => unimplemented!("dst op {:?}", op),
        }
    }

    // Create basic blocks based on mapped labels
    fn create_blocks(&self, builder: &mut InstructionBuilder) {}
}
