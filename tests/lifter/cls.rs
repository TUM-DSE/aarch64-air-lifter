use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Count leading sign bits
fn test() {
    let bytes = [
        0x41, 0x14, 0xC0, 0xDA, // cls x1, x2
        0x41, 0x14, 0xC0, 0x5A, // cls w1, w2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
