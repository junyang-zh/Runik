//! App for binary apps

use crate::kernel_stack;
//use core::arch::asm;
use core::slice;
use core::mem;
use xmas_elf::ElfFile;

pub struct App<'a> {
    pub elf_file: ElfFile<'a>,
}

impl App<'_> {
    pub fn load_from_img() -> Self {
        unsafe {
            extern "C" {
                fn app_start_addr();
                fn app_end_addr();
            }
            // Read u64 addresses since we stored them in .quad values
            let start: usize = (app_start_addr as u64 as *const u64).read_volatile().try_into().unwrap();
            let end: usize = (app_end_addr as u64 as *const u64).read_volatile().try_into().unwrap();
            // Parse ELF
            let buf: &[u8] = slice::from_raw_parts(start as *const u8, (end - start) / mem::size_of::<u8>());
            let elf = ElfFile::new(buf).unwrap();
            App {
                elf_file: elf,
            }
        }
    }

    pub fn get_entry_point(&self) -> usize {
        self.elf_file.header.pt2.entry_point().try_into().unwrap()
    }

    pub fn run(&self, user_stack_base: usize) {
        let entry_point = self.get_entry_point();
        unsafe {
            extern "C" {
                fn __restore(cx_addr: usize);
            }
            __restore(kernel_stack::push_context(crate::arch::trap::TrapContext::app_init_context(
                entry_point,
                user_stack_base,
            )) as *const _ as usize);
        }
    }
}
