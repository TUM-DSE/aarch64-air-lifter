use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
#[ignore]
// Add with carry, setting flags
fn test() {
    let bytes = [
        0x01, 0x00, 0x02, 0xBA, // adcs x1, x0, x2
        0x01, 0x00, 0x02, 0x3A, // adcs w1, w0, w2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#)
}
