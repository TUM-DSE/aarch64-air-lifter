use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
#[ignore]
// Arithmetic shift right
fn test() {
    let bytes = [
        0x41, 0xfc, 0x42, 0x93, // asr x1, x2, #2
        0x83, 0x7c, 0x1f, 0x13, // asr w3, w4, #31
        0x21, 0x28, 0xc2, 0x91, // asr x1, x1, x2
        0x60, 0x28, 0xc4, 0x1a, // asr w0, w3, w4
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#)
}
