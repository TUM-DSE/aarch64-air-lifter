use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Add shifted register
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0x10, // adr x0, pc
        0x21, 0x00, 0x00, 0x10, // adr x1, pc+1
        0xC0, 0xFF, 0xFF, 0x10, // adr x0, pc-2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
