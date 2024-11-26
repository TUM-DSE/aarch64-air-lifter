use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Compare and branch on zero
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0x34, // cbz w0, current_address
        0x20, 0x00, 0x00, 0xB4, // cbz x0, current_address + 1
        0xC1, 0xFF, 0xFF, 0xB4, // cbz x1, current_address - 2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();
    println!("{}", blob.display());
}
