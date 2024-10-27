use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[ignore]
#[test]
// Add extended and scaled register, setting flags
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0x01, // adr x0, _label with _label == pc
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#);
}
