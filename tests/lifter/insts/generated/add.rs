#![cfg_attr(rustfmt, rustfmt_skip)]// ⚠️ Automatically generated file, do not edit! ⚠️

use crate::lifter::yaml_tests::run_test_from_yaml;

#[test]
pub fn test_add_1() {
    run_test_from_yaml("tests/lifter/insts/tests/add.yaml", "add_1");
}
#[test]
pub fn test_add_2() {
    run_test_from_yaml("tests/lifter/insts/tests/add.yaml", "add_2");
}
#[test]
pub fn test_add_3() {
    run_test_from_yaml("tests/lifter/insts/tests/add.yaml", "add_3");
}
