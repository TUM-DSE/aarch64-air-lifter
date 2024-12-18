use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Compare and branch on nonzero
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0xB5, // cbnz x0, pc
        0xE0, 0xFF, 0xFF, 0x35, // cbnz w0, pc-1
        0x20, 0x80, 0x00, 0xB5, // cbnz x0, pc+1
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
