use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[ignore]
#[test]
// Add extended and scaled register, setting flags
fn test() {
    let bytes = [
        0x21, 0x00, 0x00, 0xAB, // ADDS x1, x1, x0
        0x21, 0x00, 0x00, 0x2B, // ADDS w1, w1, w0
        0x02, 0x08, 0x41, 0x2B, // ADDS w2, w0, w1, LSR#2
        0x02, 0xC8, 0x21, 0x2B, // ADDS w2, w0, w1, SXTW#2
        0x62, 0x88, 0x40, 0xB1, // ADDS x2, x3, #34, LSL #12
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#);
}
