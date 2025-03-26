//! Machine code to AIR lifting module.
//! This module is responsible for translating machine code and proofs to AIR, including parsing
//! proofs.
#![deny(missing_docs)]

use tnj::air::instructions::CodeRegion;

pub mod arm64;

/// A lifter.
pub trait Lifter {
    /// Error type when lifting fails.
    type E;

    /// Lift from a reader reading machine code and one reading proofs to a CodeRegion.
    fn lift(&self, code: &[u8], proofs: &[u8]) -> Result<CodeRegion, Self::E>;
}
