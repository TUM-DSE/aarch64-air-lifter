use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

use filecheck::{CheckerBuilder, Value};

use super::simple_variable_map::SimpleVariableMap;

const VARIABLES: [(&str, Value); 1] = [(
    "VAR_NAME",
    Value::Regex(std::borrow::Cow::Borrowed("v[0-9]+")),
)];

pub fn check_instruction(
    bytes: [u8; 4],
    directives: &str,
    variable_map: Option<SimpleVariableMap>,
) -> bool {
    let lifter = AArch64Lifter;
    let blob = lifter.lift(&bytes, &[]).unwrap();
    let result = blob.display().to_string();
    println!("{}", result);

    let mut variable_map = variable_map.unwrap_or_default();
    for (var_name, value) in VARIABLES {
        variable_map.insert(var_name.to_string(), value);
    }
    let mut checker_builder = CheckerBuilder::new();
    let checker_builder = checker_builder
        .text(directives)
        .expect("Failed to create checker builder");
    let checker = checker_builder.finish();

    checker
        .check(&result, &variable_map)
        .expect("Filecheck failed")
}
