use crate::arm64::LabelResolver;
use crate::Lifter;
use std::io::Cursor;
use target_lexicon::{Aarch64Architecture, Architecture};
use thiserror::Error;
use tnj::air::instructions::builder::InstructionBuilder;
use tnj::air::instructions::CodeRegion;
use tnj::arch::get_arch;
use tnj::pcc;
use tnj::pcc::Proof;
use tnj::sym::{Expr, TypedExprPool};
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{ARMv8, DecodeError, InstDecoder};

mod conditions;
mod flags;
mod insts;
mod operands;
mod regs;

/// A lifter for AArch64
pub struct AArch64Lifter<'a> {
    code: &'a [u8],
    proofs: &'a [u8],
}

const INSTRUCTION_SIZE: u64 = 4;

enum Flag {
    N,
    Z,
    C,
    V,
}

impl AArch64Lifter<'_> {
    /// Disassemble code and print to a string.
    pub fn disassemble<W>(&self, w: &mut W) -> Result<(), AArch64DisassemblerError>
    where
        W: ?Sized + std::io::Write,
    {
        let decoder = <ARMv8 as Arch>::Decoder::default();
        let mut reader = U8Reader::new(self.code);
        let (proof, exprs) = self.parse_proofs()?.unwrap_or_default();

        let mut pc = 0u64;

        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    let constraints = proof.constraints.get(&pc);

                    Self::print_assertions(w, &exprs, constraints.map(|c| c.asserts()), "assert")?;
                    writeln!(w, "0x{:0>4x}:\t{}", pc, inst)?;
                    Self::print_assertions(w, &exprs, constraints.map(|c| c.ensures()), "ensure")?;

                    pc += INSTRUCTION_SIZE;
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64DisassemblerError::DecodeError(e)),
            }
        }

        Ok(())
    }

    fn print_assertions<W>(
        w: &mut W,
        exprs: &TypedExprPool,
        assertions: Option<&[Expr]>,
        name: &str,
    ) -> Result<(), std::io::Error>
    where
        W: ?Sized + std::io::Write,
    {
        if let Some(asserts) = assertions {
            write!(w, "{name} {{ ")?;

            for &assert in asserts {
                write!(w, "{}; ", exprs.display(assert))?;
            }

            writeln!(w, "}}")?;
        }

        Ok(())
    }

    fn parse_proofs(&self) -> Result<Option<(Proof, TypedExprPool)>, pcc::read::Error> {
        if !self.proofs.is_empty() {
            Ok(Some(pcc::read::read(&mut Cursor::new(&self.proofs))?))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Lifter<'a> for AArch64Lifter<'a> {
    type E = AArch64LifterError;

    fn new(code: &'a [u8], proofs: &'a [u8]) -> Self {
        Self { code, proofs }
    }

    fn lift(&self) -> Result<CodeRegion, Self::E> {
        let arch = get_arch(Architecture::Aarch64(Aarch64Architecture::Aarch64)).unwrap();

        let (proof, exprs) = self.parse_proofs()?.unwrap_or_default();
        let mut code_region = CodeRegion::with_exprs(arch, exprs);

        let state = LifterState::new(&mut code_region, self.code, proof)?;

        state.lift()?;

        Ok(code_region)
    }
}

/// Private lifter tate
struct LifterState<'a> {
    builder: InstructionBuilder<'a>,
    label_resolver: LabelResolver,
    decoder: InstDecoder,
    reader: U8Reader<'a>,
    proof: Proof,
}

impl<'a> LifterState<'a> {
    fn new(
        code_region: &'a mut CodeRegion,
        code: &'a [u8],
        proof: Proof,
    ) -> Result<Self, AArch64LifterError> {
        let builder = code_region.insert();
        let decoder = <ARMv8 as Arch>::Decoder::default();
        let reader = U8Reader::new(code);
        let label_resolver = LabelResolver::new(code, &decoder)?;

        Ok(Self {
            builder,
            label_resolver,
            decoder,
            reader,
            proof,
        })
    }

    fn lift(mut self) -> Result<(), AArch64LifterError> {
        self.label_resolver.create_blocks(&mut self.builder);

        let mut pc = 0u64;

        loop {
            match self.decoder.decode(&mut self.reader) {
                Ok(inst) => {
                    let block = self.label_resolver.get_block(pc);
                    if let Some(block) = block {
                        self.builder.jump(block, vec![]);
                        self.builder.set_insert_block(block);
                    }

                    self.lift_inst(pc, inst)?;
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64LifterError::DecodeError(e)),
            }

            pc += INSTRUCTION_SIZE;
        }

        Ok(())
    }

    fn mark_next_block_as_entry(&mut self, pc: u64) {
        let next_pc = pc + INSTRUCTION_SIZE;
        if let Some(block) = self.label_resolver.get_block(next_pc) {
            self.builder.mark_entry_block(block);
        }
    }
}

/// Whether reg 31 refers to register sp or reg zero
enum SpOrZrReg {
    Sp,
    Zr,
}

/// Error type for lifting from machine code to AIR
#[derive(Debug, Error)]
pub enum AArch64LifterError {
    /// Error decoding the instructions
    #[error("Error decoding machine code: {0}")]
    DecodeError(#[from] DecodeError),

    /// Custom error with message
    #[error("{0}")]
    CustomError(String),

    /// Proof decode error
    #[error("Error decoding pcc proofs: {0}")]
    Pcc(#[from] pcc::read::Error),
}

/// Error type for disassembling from machine code to AIR
#[derive(Debug, Error)]
pub enum AArch64DisassemblerError {
    /// Error decoding the instructions
    #[error("Error decoding machine code: {0}")]
    DecodeError(#[from] DecodeError),

    /// I/O error
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Proof decode error
    #[error("Error decoding pcc proofs: {0}")]
    Pcc(#[from] pcc::read::Error),
}
