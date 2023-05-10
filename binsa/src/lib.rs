//! This is the library to dissasemble binaries and look for system calls.

use xmas_elf::{
    sections::ShType,
    ElfFile
};
use std::vec::Vec;
use std::fs;

pub mod riscv64;

pub fn elf_syscalls(file_name: &str) -> Vec<usize> {
    let elf_file = fs::read(file_name).unwrap();
    let elf = ElfFile::new(&elf_file).unwrap();
    let magic = elf.header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "Invalid elf!");
    let mut result: Vec<usize> = vec![];
    for sec in elf.section_iter() {
        if sec.get_type().unwrap() != ShType::Null && sec.get_name(&elf).unwrap() == ".text" {
            result.extend(riscv64::disasm_syscalls(sec.raw_data(&elf)));
        }
    }
    result
}
