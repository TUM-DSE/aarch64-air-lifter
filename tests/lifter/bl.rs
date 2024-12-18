use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Branch
fn test() {
    let bytes = [
        0x02, 0x00, 0x00, 0x94, // bl, pc
        0x01, 0x00, 0x00, 0x94, // bl, pc+1
        0xfe, 0xff, 0xff, 0x97, // bl, pc-2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
