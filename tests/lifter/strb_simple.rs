use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
fn test() {
    let bytes = [
        0xE0, 0x33, 0x00, 0x39, // strb w0, [sp, #12]
        0x20, 0x08, 0x00, 0x39, // strb w0, [x1]
        0x21, 0xD8, 0x21, 0x38, // strb w1, [x1, w1, sxtw #0]
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
