use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

#[test]
// Branch
fn test() {
    let bytes = [
        0x00, 0x00, 0x00, 0x14, // b _label where address of _label is 0x0
    ];

    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();
    let expected = r#"aarch64:
  ptr: i64
  regs:
    x0: i64
    x1: i64
    x2: i64
    x3: i64
    x4: i64
    x5: i64
    x6: i64
    x7: i64
    x8: i64
    x9: i64
    x10: i64
    x11: i64
    x12: i64
    x13: i64
    x14: i64
    x15: i64
    x16: i64
    x17: i64
    x18: i64
    x19: i64
    x20: i64
    x21: i64
    x22: i64
    x23: i64
    x24: i64
    x25: i64
    x26: i64
    x27: i64
    x28: i64
    x29: i64
    x30: i64
    sp: i64
    pc: i64
    n: i1
    z: i1
    c: i1
    v: i1

entry(v0: i64, v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64, v9: i64, v10: i64, v11: i64, v12: i64, v13: i64, v14: i64, v15: i64, v16: i64, v17: i64, v18: i64, v19: i64, v20: i64, v21: i64, v22: i64, v23: i64, v24: i64, v25: i64, v26: i64, v27: i64, v28: i64, v29: i64, v30: i64, v31: i64, v32: i64, v33: i1, v34: i1, v35: i1, v36: i1):
  jump block_0

block_0:

block_4:

"#;

    assert_eq!(blob.display().to_string(), expected);
}
