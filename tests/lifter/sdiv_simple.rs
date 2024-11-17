use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Signed divide
fn test() {
    let bytes = [
        0x20, 0x0C, 0xC2, 0x9A, // sdiv x0, x1, x2
        0x20, 0x0C, 0xC2, 0x1A, // sdiv w0, w1, w2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
