use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
#[ignore]
// Bitwise AND
fn test() {
    let bytes = [
        0x20, 0x04, 0x00, 0x12, // and w0, w1, #3
        0x41, 0x00, 0x40, 0x92, // and x1, x2, #1
        0x20, 0x10, 0x82, 0x8A, // and x0, x1, x2, ASR#4
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#)
}
