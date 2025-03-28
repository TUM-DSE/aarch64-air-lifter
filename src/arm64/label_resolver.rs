use super::helper;
use std::cmp::Ord;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::{BinaryHeap, HashSet};
use std::hash::Hash;
use tnj::air::instructions::builder::InstructionBuilder;
use tnj::air::instructions::BasicBlock;
use yaxpeax_arch::{Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{DecodeError, InstDecoder, Opcode};

use super::AArch64LifterError;

/// Create basic blocks for the InstructionBuilder based off labels
pub struct LabelResolver {
    checkpoints: UniqueHeap<Reverse<u64>>,
    blocks: HashMap<u64, BasicBlock>,
}

#[derive(PartialEq, Eq, Debug)]
enum CheckpointType {
    Conditional,
    Branch,
    DynamicJump,
}

impl LabelResolver {
    /// Create a new LabelResolver
    pub fn new(code: &[u8], decoder: &InstDecoder) -> Result<Self, AArch64LifterError> {
        let mut resolver = Self {
            checkpoints: UniqueHeap::new(),
            blocks: HashMap::new(),
        };

        resolver.get_checkpoints(code, decoder)?;

        Ok(resolver)
    }

    /// Create basic blocks based off labels
    pub fn resolve(
        &mut self,
        code: &[u8],
        builder: &mut InstructionBuilder,
        decoder: &InstDecoder,
    ) -> Result<(), AArch64LifterError> {
        self.get_checkpoints(code, decoder)?;
        self.create_blocks(builder);
        Ok(())
    }

    /// Get a block by address
    pub fn get_block(&self, addr: u64) -> Option<BasicBlock> {
        self.blocks.get(&addr).copied()
    }

    /// Store all addresses of branch-destinations or of instructions after branch-instructions
    fn get_checkpoints(
        &mut self,
        code: &[u8],
        decoder: &InstDecoder,
    ) -> Result<(), AArch64LifterError> {
        const INSTRUCTION_SIZE: u64 = 4;
        let mut reader = U8Reader::new(code);
        let mut address: u64 = 0;
        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    let imm: Option<(i64, CheckpointType)> = match inst.opcode {
                        Opcode::B | Opcode::BL | Opcode::Bcc(_) => Some((
                            helper::get_pc_offset_as_int(inst.operands[0]),
                            CheckpointType::Branch,
                        )),
                        Opcode::CBNZ | Opcode::CBZ => Some((
                            helper::get_pc_offset_as_int(inst.operands[1]),
                            CheckpointType::Branch,
                        )),
                        Opcode::TBNZ | Opcode::TBZ => Some((
                            helper::get_pc_offset_as_int(inst.operands[2]),
                            CheckpointType::Branch,
                        )),
                        Opcode::CCMP
                        | Opcode::CCMN
                        | Opcode::CSINC
                        | Opcode::CSINV
                        | Opcode::CSEL
                        | Opcode::CSNEG
                        | Opcode::SBFM
                        | Opcode::UBFM
                        | Opcode::BFM => Some((0, CheckpointType::Conditional)),
                        Opcode::BLR | Opcode::BR => Some((0, CheckpointType::DynamicJump)),
                        _ => None,
                    };
                    if let Some((imm, checkpoint_type)) = imm {
                        self.checkpoints.push(Reverse(address + INSTRUCTION_SIZE));
                        if checkpoint_type == CheckpointType::Branch {
                            let jump_address = imm.wrapping_add(address as i64) as u64;
                            self.checkpoints.push(Reverse(jump_address));
                        }
                    }
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }
            address += INSTRUCTION_SIZE;
        }
        Ok(())
    }

    /// Create basic blocks based checkpoints
    pub fn create_blocks(&mut self, builder: &mut InstructionBuilder) {
        while !self.checkpoints.is_empty() {
            let checkpoint = match self.checkpoints.pop() {
                Some(c) => c.0,
                None => break,
            };
            let name = helper::get_block_name(checkpoint);
            let b = builder.create_block(name.clone(), []);
            self.blocks.insert(checkpoint, b);
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
