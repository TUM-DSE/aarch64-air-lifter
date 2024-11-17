use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Multiply-add
fn test() {
    let bytes = [
        0x20, 0x0c, 0x02, 0x9B, // madd x0, x1, x2, x3
        0x20, 0x0c, 0x02, 0x1B, // madd w0, w1, w2, w3
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
