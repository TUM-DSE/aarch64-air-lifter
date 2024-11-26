use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Branch
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0x94, // bl to current_address
        0x01, 0x00, 0x00, 0x94, // bl to current_address + 1
        0xfe, 0xff, 0xff, 0x97, // bl to current_address - 2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
