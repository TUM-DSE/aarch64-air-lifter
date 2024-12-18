use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// bit-wise exclusive OR
fn test() {
    let bytes = [
        0x20, 0x00, 0x7E, 0xD2, // eor x0, x1, #0x4
        0x41, 0x00, 0x1D, 0x52, // eor w1, w2, #0x8
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
