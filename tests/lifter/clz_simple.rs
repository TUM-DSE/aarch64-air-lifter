use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Counting leading zeroes
fn test() {
    let bytes = [
        0x41, 0x10, 0xC0, 0xDA, // clz x1, x2
        0x41, 0x10, 0xC0, 0x5A, // clz w1, w2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
