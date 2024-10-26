use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
#[ignore]
// Add with carry
fn test() {
    let bytes = [
        0x10, 0x00, 0x00, 0x4D, // adc x1, x1, w0
        0x10, 0x00, 0x00, 0x0D, // adc w1, w1, w0
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#)
}
