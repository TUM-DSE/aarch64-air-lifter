use core::panic;
use std::fs;
use std::path::Path;

use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;
use elf::abi::STT_FUNC;
use elf::endian::AnyEndian;
use elf::ElfBytes;

fn read_cwasm_file(path: impl AsRef<Path>) {
    let lifter = AArch64Lifter;

    let file_data = std::fs::read(path).expect("Could not read file.");
    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).expect("Open test1");
    let abi_shdr = file
        .section_header_by_name(".text")
        .expect("Get .text section")
        .expect("Get .text section");
    let (bytes, _) = file
        .section_data(&abi_shdr)
        .expect("Get .text section data");

    let (string_table, _) = file
        .symbol_table()
        .expect("Get symbol table")
        .expect("Get symbol table");

    // filter using functions
    string_table
        .iter()
        .filter(|s| s.st_symtype() == STT_FUNC)
        .for_each(|s| {
            match lifter.lift(
                &bytes[s.st_value as usize..(s.st_value + s.st_size) as usize],
                &[],
            ) {
                Ok(blob) => {
                    let result = blob.display().to_string();
                    println!("{}", result);
                }
                Err(e) => panic!("Error lifting {:?}: {}", s, e),
            }
        });
}

#[ignore]
#[test]
fn check_missing_instructions() {
    let subdir_path = "tests/completeness/bin";

    for entry in fs::read_dir(subdir_path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "cwasm" {
                    read_cwasm_file(&path);
                }
            }
        }
    }
}
