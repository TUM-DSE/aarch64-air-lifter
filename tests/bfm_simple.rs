use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[ignore]
#[test]
// Bitfield move
fn test() {
    let bytes = [
        0x20, 0x18, 0x0C, 0x33, // bfm w0, w1, #12, #6
        0x20, 0x30, 0x06, 0x33, // bfm w0, w1, #6, #12
        0x20, 0x0C, 0x44, 0xB3, // bfm x0, x1, #4, #3
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    assert_eq!(blob.display().to_string(), r#""#);
}
