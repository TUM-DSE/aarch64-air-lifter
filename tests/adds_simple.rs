use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[ignore]
#[test]
// Add extended and scaled register, setting flags
fn test() {
    let bytes = [
        0x10, 0x00, 0x00, 0x55, // ADDS x1, x1, x0
        0x10, 0x00, 0x00, 0x15, // ADDS w1, w1, w0
        0x01, 0x04, 0x20, 0x15, // ADDS w2, w0, w1, LSR#2
        0x01, 0x64, 0x10, 0x15, // ADDS w2, w0, w1, SXTW#2
        0x31, 0x44, 0x20, 0x58, // ADDS x2, x3, #34, LSL #12
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#);
}
