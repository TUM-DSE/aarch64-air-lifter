use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Multiply-sub
fn test() {
    let bytes = [
        0x20, 0x8C, 0x02, 0x9B, // msub x0, x1, x2, x3
        0x20, 0x8C, 0x02, 0x1B, // msub w0, w1, w2, w3
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
