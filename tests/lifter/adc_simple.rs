use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Add with carry
fn test() {
    let bytes = [
        0x21, 0x00, 0x00, 0x9A, // adc x1, x1, x0
        0x21, 0x00, 0x00, 0x1A, // adc w1, w1, w0
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
