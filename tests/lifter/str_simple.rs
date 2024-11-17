use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Store register
fn test() {
    let bytes = [
        0x00, 0x24, 0x00, 0xF8, // str x0, [x0], #2
        0x00, 0x14, 0x00, 0xB8, // str w0, [x0], #1
        0x41, 0x68, 0x22, 0xF8, // str x1, [x2, x2, lsl #0]
        0x21, 0x78, 0x23, 0xB8, // str w1, [x1, x3, lsl #2]
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
