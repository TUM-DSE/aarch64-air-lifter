#![cfg_attr(rustfmt, rustfmt_skip)]// ⚠️ Automatically generated file, do not edit! ⚠️

use crate::lifter::yaml_tests::run_test_from_yaml;

#[test]
pub fn test_movk_1() {
    run_test_from_yaml("tests/lifter/insts/tests/movk.yaml", "movk_1");
}
#[test]
pub fn test_movk_2() {
    run_test_from_yaml("tests/lifter/insts/tests/movk.yaml", "movk_2");
}
