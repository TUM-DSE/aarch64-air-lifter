use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
fn test() {
    let bytes = [
        0x21, 0x00, 0x00, 0xCB, // sub x1, x1, x0
        0x21, 0x00, 0x00, 0x4B, // sub w1, w1, w0
        0x02, 0xC0, 0x21, 0x4B, // sub w2, w0, w1, SXTW
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
