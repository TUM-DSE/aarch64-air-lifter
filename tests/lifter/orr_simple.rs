use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Bitwise AND
fn test() {
    let bytes = [
        0x20, 0x04, 0x00, 0x32, // orr w0, w1, #0x3
        0x41, 0x00, 0x40, 0xB2, // orr x1, x2, #0x1
        0x20, 0x10, 0x02, 0xAA, // orr x0, x1, x2, lsl#4
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
