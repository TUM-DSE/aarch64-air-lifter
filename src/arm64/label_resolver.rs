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
    checkpoints: BinaryHeap<usize>,
}

impl LabelResolver {
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
        let mut address: usize = 0;
        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    println!("{}", inst);
                    match inst.opcode {
                        Opcode::B => {
                            self.checkpoints.push(address + 1);
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }
            address += 1;
        }
        Ok(())
    }

    fn get_immediate(&self, operand: Operand) -> Result<usize, AArch64LifterError> {
        let imm = match operand {
            Operand::Imm16(imm) => imm as usize,
            Operand::Imm64(imm) => imm as usize,
            Operand::Immediate(imm) => imm as usize,
            Operand::ImmShift(imm, shift) => ((imm as usize) << shift),
            Operand::ImmShiftMSL(imm, shift) => {
                let imm = (imm as usize) << shift;
                let mask = (1 << shift) - 1;
                imm & mask
            }
            Operand::ImmediateDouble(imm) => imm.floor() as usize,
            _ => return Err(AArch64LifterError::DecodeError(DecodeError::InvalidOperand)),
        };
        Ok(imm)
    }

    // Create basic blocks based on mapped labels
    fn create_blocks(&self, builder: &mut InstructionBuilder) {}
}
