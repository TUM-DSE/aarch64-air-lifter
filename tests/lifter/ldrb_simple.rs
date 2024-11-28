use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Load register byte (register)
fn test() {
    let bytes = [
        0xE0, 0xDB, 0x62, 0x38, // ldrb w0, [sp, w2, sxtw #0]
        0x20, 0xD8, 0x62, 0x38, // ldrb w0, [x1, w2, sxtw #0]
        0x20, 0x58, 0x62, 0x38, // ldrb w0, [x1, w2, uxtw #0]
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();

    println!("{}", blob.display());
}
