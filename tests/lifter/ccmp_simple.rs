use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Conditional Compare (register) sets the value of the condition flags to the result of the comparison of two registers if the condition is TRUE, and an immediate value otherwise.
fn test() {
    let bytes = [
        0x03, 0x00, 0x41, 0xFA, // ccmp x0, x1, #0x3, eq
        0x49, 0x50, 0x43, 0xFA, // ccmp x2, x3, #0x9, pl
        0x04, 0x60, 0x42, 0x7A, // ccmp w0, w2, #0x4, vs
        0xC0, 0xE0, 0x46, 0xFA, // ccmp x6, x6, #0x0, al
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
