//! Machine code to AIR lifting module.
//! This module is responsible for translating machine code and proofs to AIR, including parsing
//! proofs.
#![deny(missing_docs)]

use tnj::air::instructions::CodeRegion;

pub mod arm64;

/// A lifter.
pub trait Lifter<'a> {
    /// Error type when lifting fails.
    type E;

    /// Construct a new lifter.
    fn new(code: &'a [u8], proofs: &'a [u8]) -> Self;

    /// Lift from a reader reading machine code and one reading proofs to a CodeRegion.
    fn lift(&self) -> Result<CodeRegion, Self::E>;
}
