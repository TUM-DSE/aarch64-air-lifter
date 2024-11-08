#![allow(unused)]

use std::collections::BinaryHeap;
use tnj::air::instructions::{builder::InstructionBuilder, BlockParamData};
use yaxpeax_arch::{Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{DecodeError, InstDecoder, Opcode, Operand};

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

    // Create basic blocks based off labels
    fn _resolve(&mut self, code: &[u8], builder: &mut InstructionBuilder, decoder: &InstDecoder) {
        self.get_checkpoints(code, decoder);
        self.create_blocks(builder);
    }

    // Store all addresses of branch-destinations or of instructions after branch-instructions
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
                            // TODO: Uses dynamic address stored in register. Might need to be handled in the future differently
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

    // Create basic blocks based checkpoints
    fn create_blocks(&mut self, builder: &mut InstructionBuilder) {
        while !self.checkpoints.is_empty() {
            let checkpoint = match self.checkpoints.pop() {
                Some(c) => c,
                None => break,
            };
            let name = format!("block_{}", checkpoint);
            builder.create_block(name, Vec::<BlockParamData>::new());
        }
    }
}
