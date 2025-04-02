use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;

use filecheck::{CheckerBuilder, Value};

use super::simple_variable_map::SimpleVariableMap;

const VARIABLES: [(&str, Value); 2] = [
    (
        "VAR_NAME",
        Value::Regex(std::borrow::Cow::Borrowed("v[0-9]+")),
    ),
    (
        "LABEL",
        Value::Regex(std::borrow::Cow::Borrowed("[a-zA-Z0-9_]+")),
    ),
];

pub struct CheckInstructionArgs {
    pub variable_map: SimpleVariableMap,
    pub print_to_std: bool,
    pub debug: bool,
}

impl CheckInstructionArgs {
    pub fn new(variable_map: SimpleVariableMap, print_to_std: bool, debug: bool) -> Self {
        Self {
            variable_map,
            print_to_std,
            debug,
        }
    }
}

impl Default for CheckInstructionArgs {
    fn default() -> Self {
        Self::new(SimpleVariableMap::default(), true, false)
    }
}

pub fn check_instruction(
    bytes: &[u8],
    proofs: Option<&[u8]>,
    directives: &str,
    args: CheckInstructionArgs,
) -> bool {
    let lifter = AArch64Lifter::new(bytes, proofs.unwrap_or(&[]));
    let code_region = lifter.lift().unwrap();
    let result = code_region.display().to_string();
    if args.debug {
        let blocks_count = code_region.blocks().len();
        let inst_count = code_region
            .blocks()
            .iter()
            .fold(0, |acc, b| acc + b.inst_count());
        println!("Blocks: {}, Instructions: {}", blocks_count, inst_count);
    }
    if args.print_to_std {
        println!("{}", result);
    }

    let mut variable_map = args.variable_map;
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
