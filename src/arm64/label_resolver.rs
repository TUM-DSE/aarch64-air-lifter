#![allow(unused)]

use std::cmp::Ord;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::hash::Hash;
use tnj::air::instructions::{builder::InstructionBuilder, BlockParamData};
use yaxpeax_arch::{Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{DecodeError, InstDecoder, Opcode, Operand};

use super::AArch64LifterError;

/// Create basic blocks for the InstructionBuilder based off labels
pub struct LabelResolver {
    checkpoints: UniqueHeap<Reverse<isize>>,
}

impl Default for LabelResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelResolver {
    /// Create a new LabelResolver
    pub fn new() -> Self {
        Self {
            checkpoints: UniqueHeap::new(),
        }
    }

    /// Create basic blocks based off labels
    pub fn resolve(
        &mut self,
        code: &[u8],
        builder: &mut InstructionBuilder,
        decoder: &InstDecoder,
    ) {
        self.get_checkpoints(code, decoder);
        self.create_blocks(builder);
    }

    // Store all addresses of branch-destinations or of instructions after branch-instructions
    fn get_checkpoints(
        &mut self,
        code: &[u8],
        decoder: &InstDecoder,
    ) -> Result<(), AArch64LifterError> {
        let instruction_size = 4;
        let mut reader = U8Reader::new(code);
        let mut address: isize = -instruction_size;
        loop {
            address += instruction_size;
            match decoder.decode(&mut reader) {
                Ok(inst) => {
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
                    self.checkpoints.push(Reverse(address + instruction_size));
                    let jump_address = imm + address;
                    self.checkpoints.push(Reverse(jump_address));
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }
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
                Some(c) => c.0,
                None => break,
            };
            let name = format!("block_{}", checkpoint);
            builder.create_block(name, Vec::<BlockParamData>::new());
        }
    }
}

struct UniqueHeap<T>
where
    T: Ord + Hash + Clone,
{
    heap: BinaryHeap<T>,
    set: HashSet<T>,
}

impl<T> UniqueHeap<T>
where
    T: Ord + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            set: HashSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn push(&mut self, item: T) {
        if self.set.insert(item.clone()) {
            self.heap.push(item);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.heap.pop();
        if let Some(ref item) = item {
            self.set.remove(item);
        }
        item
    }
}
