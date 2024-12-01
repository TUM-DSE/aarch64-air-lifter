use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Bitfield move
fn test() {
    let bytes = [
        0x41, 0x78, 0x0C, 0x33, // bfm w1, w2, #12, #30 <=> bfxil w1, w2, #12, #19
        0x41, 0xC8, 0x42, 0xB3, // bfm x1, x2, #2, #50 <=> bfxil x1, x2, #2, #49
        0x41, 0x04, 0x41, 0xB3, // bfm x1, x2, #1, #1 <=> bfxil x1, x2, #1, #1
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
