use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Conditional select increment
fn test() {
    let bytes = [
        0x41, 0xA0, 0x83, 0x5A, // csinv w1, w2, w3, ge
        0x41, 0xA0, 0x83, 0xDA, // csinv x1, x2, x3, ge
        0x21, 0x30, 0x82, 0xDA, // csinv x1, x1, x2, cc
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
