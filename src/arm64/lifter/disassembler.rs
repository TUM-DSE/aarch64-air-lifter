use crate::arm64::lifter::INSTRUCTION_SIZE;
use crate::arm64::{AArch64DisassemblerError, AArch64Lifter};
use std::collections::HashMap;
use std::fmt::Display;
use tnj::pcc::InstConstraint;
use tnj::sym::ExprPool;
use yaxpeax_arch::{Arch, Decoder, U8Reader};
use yaxpeax_arm::armv8::a64::{ARMv8, DecodeError};

impl AArch64Lifter {
    /// Disassemble code and print to a string.
    pub fn disassemble<W>(&self, w: &mut W, code: &[u8]) -> Result<(), AArch64DisassemblerError>
    where
        W: ?Sized + std::io::Write,
    {
        self.disassemble_with_constraints::<_, String>(
            w,
            code,
            &Default::default(),
            &Default::default(),
        )
    }

    /// Disassemble code and print to a string. This function also prints constraints associated
    /// with instructions.
    pub fn disassemble_with_constraints<W, T: Display>(
        &self,
        w: &mut W,
        code: &[u8],
        constraints: &HashMap<u64, Vec<(T, InstConstraint)>>,
        exprs: &ExprPool,
    ) -> Result<(), AArch64DisassemblerError>
    where
        W: ?Sized + std::io::Write,
    {
        let decoder = <ARMv8 as Arch>::Decoder::default();
        let mut reader = U8Reader::new(code);

        let mut pc = 0u64;

        loop {
            match decoder.decode(&mut reader) {
                Ok(inst) => {
                    let constraints = constraints.get(&pc).map(|v| &v[..]).unwrap_or(&[]);

                    constraints.iter().try_for_each(|(reg, constraints)| {
                        constraints
                            .pre()
                            .try_for_each(|c| writeln!(w, "← {}", c.display(exprs, reg)))
                    })?;

                    writeln!(w, "0x{:0>4x}:\t{}", pc, inst)?;
                    pc += INSTRUCTION_SIZE;

                    constraints.iter().try_for_each(|(reg, constraints)| {
                        constraints
                            .post()
                            .try_for_each(|c| writeln!(w, "→ {}", c.display(exprs, reg)))
                    })?;
                }
                Err(DecodeError::ExhaustedInput) => break,
                Err(e) => return Err(AArch64DisassemblerError::DecodeError(e)),
            }
        }

        Ok(())
    }
}
