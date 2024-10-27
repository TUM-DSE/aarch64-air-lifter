use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
#[ignore]
// Absolute value
fn test() {
    let bytes = [
        0x41, 0xB8, 0xE0, 0x5E, // abs d1, d2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#)
}
