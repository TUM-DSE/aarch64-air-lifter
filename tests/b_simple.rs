use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[ignore]
#[test]
// Branch
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0x14, // b _label where address of _label is 0x0
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#);
}
