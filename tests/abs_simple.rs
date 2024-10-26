use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
#[ignore]
// Add shifted register
fn test() {
    let bytes = [
        0x11, 0x10, 0x60, 0x6D, // abs x1, x2
        0x00, 0x10, 0x60, 0x2D, // abs w0, w0
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#)
}
