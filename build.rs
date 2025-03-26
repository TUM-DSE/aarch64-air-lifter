use prettier_please::unparse;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{fs, path::Path};
use syn::File;

#[derive(Deserialize)]
struct TestFile {
    tests: Vec<TestSpec>,
}

#[derive(Deserialize)]
struct TestSpec {
    name: String,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tests/lifter/insts");

    let tests_dir = Path::new("tests/lifter/insts/tests");
    let out_dir = Path::new("tests/lifter/insts/generated");
    // let out_dir = env::var_os("OUT_DIR").unwrap();

    if !out_dir.exists() {
        fs::create_dir_all(out_dir).unwrap();
    }

    let mut mod_file_content =
        "#![cfg_attr(rustfmt, rustfmt_skip)]\n// ⚠️ Automatically generated file, do not edit! ⚠️\n\n".to_string();

    for entry in fs::read_dir(tests_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }

        let yaml_str = fs::read_to_string(&path).unwrap();
        let test_file: TestFile = serde_yaml::from_str(&yaml_str).unwrap();

        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let mut rust_tests = "#![cfg_attr(rustfmt, rustfmt_skip)]// ⚠️ Automatically generated file, do not edit! ⚠️\n\nuse crate::lifter::yaml_tests::run_test_from_yaml;\n\n".to_string();

        for test in test_file.tests {
            let test_name = &test.name;
            let test_fn_name = format_ident!("test_{}", test.name);
            let test_file = format!("tests/lifter/insts/tests/{}.yaml", file_stem);
            let ts = quote! {
                #[test]
                pub fn #test_fn_name() {
                    run_test_from_yaml(#test_file, #test_name);
                }
            };
            let file: File = syn::parse2(ts).unwrap();
            rust_tests.push_str(&unparse(&file));
        }

        // write the individual rust test file
        fs::write(out_dir.join(format!("{}.rs", file_stem)), rust_tests).unwrap();

        // append mod declaration
        mod_file_content.push_str(&format!("pub mod {};\n", file_stem));
    }

    fs::write(out_dir.join("mod.rs"), mod_file_content).unwrap();
}
