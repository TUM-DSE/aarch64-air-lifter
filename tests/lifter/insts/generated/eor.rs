#![cfg_attr(rustfmt, rustfmt_skip)]// ⚠️ Automatically generated file, do not edit! ⚠️

use crate::lifter::yaml_tests::run_test_from_yaml;

#[test]
pub fn test_eor_1() {
    run_test_from_yaml("tests/lifter/insts/tests/eor.yaml", "eor_1");
}
#[test]
pub fn test_eor_2() {
    run_test_from_yaml("tests/lifter/insts/tests/eor.yaml", "eor_2");
}
