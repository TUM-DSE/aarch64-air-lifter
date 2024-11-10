use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
#[ignore]
// Bitwise bit clear
fn test() {
    let bytes = [
        0x41, 0x00, 0x23, 0x8A, // bic x1, x2, x3
        0x20, 0x00, 0x22, 0x0A, // bic w0, w1, w2
        0x00, 0x10, 0xE1, 0x0A, // bic w0, w0, w1, ROR#4
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#)
}
