use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Add shifted register
fn test() {
    let bytes = [
        0x21, 0x00, 0x00, 0x8B, // add x1, x1, s0
        0x21, 0x00, 0x00, 0x0B, // add w1, w1, w0
        0x02, 0xc0, 0x21, 0x0B, // add w2, w0, w1, SXTW
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
