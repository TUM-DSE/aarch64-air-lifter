use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use aarch64_air_lifter::arm64::AArch64Lifter;
use aarch64_air_lifter::Lifter;
use elf::abi::STT_FUNC;
use elf::endian::AnyEndian;
use elf::ElfBytes;

fn read_elf_file(path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let file_data = std::fs::read(path)?;
    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice)?;

    let text_shdr = file
        .section_header_by_name(".text")?
        .ok_or("Missing .text section")?;

    let (bytes, _) = file.section_data(&text_shdr)?;

    let (string_table, _) = file.symbol_table()?.ok_or("Missing symbol table")?;

    let section_addr = text_shdr.sh_addr;
    let mut execution_time = Duration::from_millis(0);
    for s in string_table.iter().filter(|s| s.st_symtype() == STT_FUNC) {
        // Calculate offset relative to section start
        let offset: usize = (s
            .st_value
            .checked_sub(section_addr)
            .ok_or("Symbol address before section start")?)
        .try_into()
        .map_err(|_| "Offset too large")?;

        let size: usize = s.st_size.try_into().map_err(|_| "Symbol size too large")?;

        // Check 4-byte alignment
        if offset % 4 != 0 || size % 4 != 0 {
            println!("Warning: Symbol {:?} is not 4-byte aligned", s);
            continue;
        }

        let end = offset.checked_add(size).ok_or("Symbol range overflow")?;

        // Check bounds
        if end > bytes.len() {
            println!("Warning: Symbol {:?} extends beyond section bounds", s);
            continue;
        }

        let start = Instant::now();
        let lifter = AArch64Lifter::new();
        let panic = std::panic::catch_unwind(|| match lifter.lift(&bytes[offset..end], &[]) {
            Ok(_code_region) => Ok(()),
            Err(e) => {
                println!("Error lifting {:?}: {}", s, e);
                Err(e)
            }
        });

        if let Err(e) = panic {
            println!("Panicked while processing symbol {:?}: {:?}", s, e);
        } else {
            execution_time += start.elapsed();
        }
    }
    println!("Execution time: {:?}", execution_time);
    Ok(())
}

#[ignore]
#[test]
fn check_missing_instructions() {
    let subdir_path = "tests/bin";

    for entry in fs::read_dir(subdir_path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_file() {
            let res = read_elf_file(&path);
            if let Err(e) = res {
                panic!("Error processing {:?}: {:?}", path, e);
            }
        }
    }
}
