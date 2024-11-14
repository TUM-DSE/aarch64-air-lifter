use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
fn test() {
    let bytes = [
        0xE0, 0x03, 0x01, 0xCB, // neg x0, x1
        0xE0, 0x03, 0x01, 0x4B, // neg w0, w1
        0xE1, 0x0F, 0x01, 0xCB, // neg x1, x1, lsl #3
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
