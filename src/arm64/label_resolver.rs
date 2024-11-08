#![allow(clippy::all)]
#![allow(warnings)]

use crate::Lifter;
use std::collections::BinaryHeap;
use tnj::air::instructions::{builder::InstructionBuilder, Inst};
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{
    ARMv8, DecodeError, InstDecoder, Instruction, Opcode, Operand, ShiftStyle, SizeCode,
};

use super::AArch64LifterError;

/// Create basic blocks for the InstructionBuilder based off labels
pub struct LabelResolver {
    checkpoints: BinaryHeap<usize>,
}

impl LabelResolver {
    /*
        High-Level:
        Goes through all instructions and creates checkpoints
        A checkpoint is where a label is jumping to or where a label is

        After gathering all the checkpoints, it creates the blocks
    */

    fn _resolve(&self, code: &[u8], builder: &mut InstructionBuilder, decoder: &InstDecoder) {
        self.get_checkpoints(code, decoder);
        self.create_blocks(builder);
    }

    fn get_checkpoints(
        &self,
        code: &[u8],
        decoder: &InstDecoder,
    ) -> Result<(), AArch64LifterError> {
        let mut reader = U8Reader::new(code);

        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    println!("{}", inst);
                    match inst.opcode {
                        // Currently not supported
                        Opcode::ADD => {}
                        op => {}
                    }
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }
        }
        Ok(())
    }

    // Create basic blocks based on mapped labels
    fn create_blocks(&self, builder: &mut InstructionBuilder) {}
}
