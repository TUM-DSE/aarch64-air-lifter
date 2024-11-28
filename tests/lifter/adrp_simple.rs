use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Form PC-relative address to 4KB page
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0x90, // adrp x0, pc
        0x00, 0x00, 0x00, 0xB0, // adrp x0, pc+4096
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
