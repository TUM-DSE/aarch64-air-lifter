use std::vec;

use aarch64_air_lifter::arm64::LabelResolver;
use target_lexicon::{Aarch64Architecture, Architecture};
use tnj::air::instructions::Blob;
use tnj::arch::get_arch;
use yaxpeax_arch::Arch;
use yaxpeax_arm::armv8::a64::ARMv8;

#[test]
// Add with carry
fn test() {
    let bytes = [
        0x20, 0x00, 0x02, 0xB0, // add x0, x1, x2
        0xFF, 0xFF, 0xFF, 0x17, // b -1
        0x00, 0x00, 0x00, 0x14, // b 0
        0xFD, 0xFF, 0xFF, 0x17, // b -3
        0x41, 0x00, 0x03, 0x8B, // add x1, x2, x3
        0x02, 0x00, 0x00, 0x14, // b 2
        0x41, 0x00, 0x03, 0x8B, // add x1, x2, x3
    ];

    let arch = get_arch(Architecture::Aarch64(Aarch64Architecture::Aarch64)).unwrap();
    let mut blob = Blob::new(arch);
    let mut builder = blob.insert();
    let decoder = <ARMv8 as Arch>::Decoder::default();

    let _ = LabelResolver::new(&bytes, &mut builder, &decoder).unwrap();

    let expected: Vec<String> = vec![
        "entry", "block_0", "block_8", "block_12", "block_16", "block_24", "block_28",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut actual = vec![];
    for block in blob.blocks() {
        actual.push(block.name());
    }

    assert_eq!(expected, actual);
}
