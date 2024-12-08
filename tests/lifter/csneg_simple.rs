use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Conditional select negation
fn test() {
    let bytes = [
        0x20, 0x04, 0x82, 0xDA, // csneg x0, x1, x2, eq
        0x20, 0x04, 0x82, 0x5A, // csneg w0, w1, w2, eq
        0x62, 0xD4, 0x84, 0xDA, // csneg x2, x3, x4, le
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
