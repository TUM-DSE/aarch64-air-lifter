//! Machine code to AIR lifting module.
//! This module is responsible for translating machine code and proofs to AIR, including parsing
//! proofs.
#![deny(missing_docs)]

use air::instructions::CodeRegion;
use std::error::Error;
use target_lexicon::Architecture;

pub mod arm64;

/// A lifter.
pub trait Lifter {
    /// Error type when lifting fails.
    type E: Error;

    /// Get the architecture for this lifter
    fn arch() -> Architecture;

    /// Construct a new lifter.
    fn new() -> Self;

    /// Lift from a reader reading machine code and one reading proofs to a CodeRegion.
    fn lift(&self, code: &[u8], proofs: &[u8]) -> Result<CodeRegion, Self::E>;
}
