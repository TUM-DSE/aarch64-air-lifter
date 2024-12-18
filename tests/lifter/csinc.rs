use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Conditional select increment
fn test() {
    let bytes = [
        0x20, 0x04, 0x82, 0x9A, // csinc x0, x1, x2, EQ
        0x20, 0x04, 0x84, 0x1A, // csinc w0, w1, w2, EQ
        0x62, 0xD4, 0x84, 0x9A, // csinc x2, x3, x4, LE
        0x20, 0x00, 0x02, 0x8B, // add x0, x1, x2
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
