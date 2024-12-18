use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Conditional Select
fn test() {
    let bytes = [
        0x20, 0x20, 0x82, 0x9A, // csel x0, x1, x2, cs = hs, nlast
        0x20, 0x00, 0x82, 0x1A, // csel w0, w1, w2, eq = none
        0x20, 0x40, 0x82, 0x9A, // csel x0, x1, x2, mi = first
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
