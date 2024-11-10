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

    assert_eq!(
        blob.display().to_string(),
        r#"aarch64:
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

entry(v0: i64, v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64, v9: i64, v10: i64, v11: i64, v12: i64, v13: i64, v14: i64, v15: i64, v16: i64, v17: i64, v18: i64, v19: i64, v20: i64, v21: i64, v22: i64, v23: i64, v24: i64, v25: i64, v26: i64, v27: i64, v28: i64, v29: i64, v30: i64, v31: i64, v32: i64):
  v33 = i64.read_reg "x1"
  v34 = i64.read_reg "x0"
  v35 = i64.sub v33, v34
  i64.write_reg v35, "x1"
  v36 = i64.read_reg "x1"
  v37 = i32.trunc_i64 v36
  v38 = i64.zext_i32 v37
  v39 = i64.read_reg "x0"
  v40 = i32.trunc_i64 v39
  v41 = i64.zext_i32 v40
  v42 = i64.sub v38, v41
  v43 = i32.trunc_i64 v42
  v44 = i64.zext_i32 v43
  i64.write_reg v44, "x1"
  v45 = i64.read_reg "x0"
  v46 = i32.trunc_i64 v45
  v47 = i64.zext_i32 v46
  v48 = i64.read_reg "x1"
  v49 = i32.trunc_i64 v48
  v50 = i64.zext_i32 v49
  v51 = i32.trunc_i64 v50
  v52 = i64.sext_i32 v51
  v53 = i64.sub v47, v52
  v54 = i32.trunc_i64 v53
  v55 = i64.zext_i32 v54
  i64.write_reg v55, "x2"

"#
    );
}
