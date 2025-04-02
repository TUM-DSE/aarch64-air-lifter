use crate::common::lib::{check_instruction, CheckInstructionArgs};
use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};
use std::{env, fs};

#[derive(Deserialize)]
struct TestFile {
    tests: Vec<TestSpec>,
}

#[derive(Deserialize)]
struct TestSpec {
    name: String,
    bytes: Vec<u8>,
    directives: String,
    proofs: Option<Vec<u8>>,
    skip: Option<bool>,
}

static FIX_LOCK: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(Default::default()));

pub fn run_test_from_yaml(file: &str, test_name: &str) {
    let yaml_str = fs::read_to_string(file).expect("Cannot read YAML file");
    let mut test_file: TestFile = serde_yaml::from_str(&yaml_str).expect("Invalid YAML");

    let fix_tests = env::var("FIX_TESTS").is_ok();

    if fix_tests {
        // we only want a single instance to update our tests.
        let mut lock = FIX_LOCK.lock().unwrap();
        if !lock.insert(file.to_string()) {
            return;
        }
        drop(lock);

        println!("Updating directives for '{}'", file);
        for test in test_file.tests.iter_mut() {
            if let Some(true) = test.skip {
                continue;
            }
            let lifter = AArch64Lifter::new(
                &test.bytes,
                test.proofs
                    .as_ref()
                    .map(|bytes| &bytes[..])
                    .unwrap_or_default(),
            );
            let blob = lifter.lift().expect("Lifter failed");
            let result = blob.display().to_string();

            // Reconstruct directives with 'check' and 'nextln'
            let new_directives = result
                .lines()
                .skip_while(|l| !l.contains("entry"))
                .fold((String::new(), true), |(mut acc, is_start), ln| {
                    if ln.is_empty() {
                        (acc, true)
                    } else {
                        if !acc.is_empty() {
                            acc.push('\n');
                        }
                        if is_start {
                            acc.push_str("check: ");
                        } else {
                            acc.push_str("nextln: ");
                        }
                        acc.push_str(ln);
                        (acc, false)
                    }
                })
                .0;

            test.directives = new_directives;
        }

        // Save the updated YAML back to the file
        let updated_yaml = pretty_print_yaml(&test_file);
        fs::write(file, updated_yaml).expect("Failed to write YAML file");
    } else {
        let test = test_file
            .tests
            .iter_mut()
            .find(|t| t.name == test_name)
            .unwrap_or_else(|| panic!("Test '{}' not found in '{}'", test_name, file));

        if let Some(true) = test.skip {
            return;
        }

        assert!(
            check_instruction(
                &test.bytes,
                test.proofs.as_ref().map(|bytes| &bytes[..]),
                &test.directives,
                CheckInstructionArgs::default()
            ),
            "Test '{}' failed",
            test_name
        );
    }
}

fn pretty_print_yaml(test_file: &TestFile) -> String {
    let mut s = String::new();

    s.push_str("tests:\n");
    test_file.tests.iter().for_each(
        |TestSpec {
             name,
             bytes,
             directives,
             proofs,
             skip,
         }| {
            let directives = directives.lines().fold(String::new(), |mut acc, rhs| {
                acc.push_str("\n      ");
                acc.push_str(rhs);
                acc
            });
            let proofs = if let Some(proofs) = proofs {
                let bytes = proofs
                    .chunks(12)
                    .map(format_bytes)
                    .reduce(|mut lhs, rhs| {
                        lhs.push_str(",\n           ");
                        lhs.push_str(&rhs);
                        lhs
                    })
                    .unwrap_or_default();
                format!("\n  proofs: [{bytes}]")
            } else {
                String::new()
            };
            let bytes = format_bytes(bytes);
            let skip = if let Some(skip) = skip {
                format!("\n  skip: {skip}")
            } else {
                String::new()
            };
            s.push_str(&format!(
                "\
- name: {name}
  bytes: [{bytes}]{skip}{proofs}
  directives: |{directives}
"
            ));
        },
    );

    s
}

fn format_bytes<'a>(chunk: impl IntoIterator<Item = &'a u8>) -> String {
    chunk
        .into_iter()
        .map(|b| format!("0x{b:02x}"))
        .reduce(|mut lhs, rhs| {
            lhs.push_str(", ");
            lhs.push_str(&rhs);
            lhs
        })
        .unwrap_or_default()
}
