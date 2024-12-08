use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// bit-wise exclusive OR
fn test() {
    let bytes = [
        0x20, 0x30, 0xC2, 0x93, // extr x0, x1, x2, #12
        0x00, 0xC8, 0xC0, 0x93, // ror x0, x0, #50 <=> extr x0, x0, x0, #50
        0x41, 0x0C, 0x83, 0x13, // extr w1, w2, w3, #3
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
