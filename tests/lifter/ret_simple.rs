use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Signed divide
fn test() {
    let bytes = [
        0xc0, 0x03, 0x5f, 0xd6, // ret
        0x20, 0x00, 0x5f, 0xd6, // ret	x1
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
