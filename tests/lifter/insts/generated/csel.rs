#![cfg_attr(rustfmt, rustfmt_skip)]// ⚠️ Automatically generated file, do not edit! ⚠️

use crate::lifter::yaml_tests::run_test_from_yaml;

#[test]
pub fn test_csel_1() {
    run_test_from_yaml("tests/lifter/insts/tests/csel.yaml", "csel_1");
}
#[test]
pub fn test_csel_2() {
    run_test_from_yaml("tests/lifter/insts/tests/csel.yaml", "csel_2");
}
#[test]
pub fn test_csel_3() {
    run_test_from_yaml("tests/lifter/insts/tests/csel.yaml", "csel_3");
}
