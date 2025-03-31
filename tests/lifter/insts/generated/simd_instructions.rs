#![cfg_attr(rustfmt, rustfmt_skip)]// ⚠️ Automatically generated file, do not edit! ⚠️

use crate::lifter::yaml_tests::run_test_from_yaml;

#[test]
pub fn test_simd_scalar() {
    run_test_from_yaml("tests/lifter/insts/tests/simd_instructions.yaml", "simd_scalar");
}
#[test]
pub fn test_simd_vector() {
    run_test_from_yaml("tests/lifter/insts/tests/simd_instructions.yaml", "simd_vector");
}
#[test]
pub fn test_simd_vector_2() {
    run_test_from_yaml(
        "tests/lifter/insts/tests/simd_instructions.yaml",
        "simd_vector_2",
    );
}
#[test]
pub fn test_simd_mov_to_scalar() {
    run_test_from_yaml(
        "tests/lifter/insts/tests/simd_instructions.yaml",
        "simd_mov_to_scalar",
    );
}
