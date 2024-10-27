use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[ignore]
#[test]
// Branch conditionally
fn test() {
    let bytes = [
        // Assumption: _label = pc
        0x00, 0x00, 0x00, 0x54, // b.eq _label
        0x08, 0x00, 0x00, 0x54, // b.hi _label
        0x0e, 0x00, 0x00, 0x54, // b.al _label
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#);
}
