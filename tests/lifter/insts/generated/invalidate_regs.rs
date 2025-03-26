#![cfg_attr(rustfmt, rustfmt_skip)]// ⚠️ Automatically generated file, do not edit! ⚠️

use crate::lifter::yaml_tests::run_test_from_yaml;

#[test]
pub fn test_test_no_invalidate_regs() {
    run_test_from_yaml(
        "tests/lifter/insts/tests/invalidate_regs.yaml",
        "test_no_invalidate_regs",
    );
}
