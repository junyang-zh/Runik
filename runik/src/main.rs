//! The main module and entrypoint
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)

//#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate cfg_if;

extern crate alloc;
extern crate goblin;

use core::arch::global_asm;

#[macro_use]
mod console;

pub mod elf;
pub mod loader;
pub mod kernel_stack;
mod kernel_panic;
mod sbi;
mod sync;
pub mod mm;
pub mod syscall;
pub mod arch;
pub mod plat;

global_asm!(include_str!("link_app.S"));

/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() -> () {
    clear_bss();
    arch::trap::init();
    mm::init();
    // mm::kernel_heap::heap_test();
    loader::init();
    let mut app_manager = loader::APP_MANAGER.exclusive_access();
    unsafe {
        let elf_data = app_manager.load_app();
        let (kernel_space, user_stack_base, entry_point) = crate::mm::addr_space::AddrSpace::new_with_elf(elf_data);
        println!("[kernel] user_sp: {:p}, entry_p: {:p}", user_stack_base as *const usize, entry_point as *const usize);
        app_manager.user_stack_base = user_stack_base;
        app_manager.entry_point = entry_point;
        kernel_space.activate();
        println!("[kernel] Paging mode activated");
        extern "C" {
            fn __restore(cx_addr: usize);
        }
        __restore(kernel_stack::push_context(crate::arch::trap::TrapContext::app_init_context(
            app_manager.entry_point,
            app_manager.user_stack_base,
        )) as *const _ as usize);
    }
    /*let kernel_space: Arc<UPIntrFreeCell<AddrSpace>> =
        Arc::new(unsafe { UPIntrFreeCell::new() });*/
}
