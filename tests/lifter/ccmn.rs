use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Conditional compare negative (immediate)
fn test() {
    let bytes = [
        0x03, 0x00, 0x41, 0xBA, // ccmn x0, x1, #0x3, eq
        0x49, 0x50, 0x43, 0xBA, // ccmn x2, x3, #0x9, pl
        0x04, 0x60, 0x42, 0x3A, // ccmn w0, w2, #0x4, vs
        0xC0, 0xE0, 0x46, 0xBA, // ccmn x6, x6, #0x0, al
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
