use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
fn test() {
    let bytes = [
        0x40, 0x44, 0x40, 0xF8, // ldr x0, [x2], #4
        0x41, 0xC4, 0x40, 0xB8, // ldr w1, [x2], #12
        0xC2, 0xFF, 0xFF, 0x18, // ldr w2, -0x8
        0xA2, 0xFF, 0xFF, 0x58, // ldr x2, -0xc
        0x41, 0x68, 0x61, 0xF8, // ldr x1, [x2, x1]
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
