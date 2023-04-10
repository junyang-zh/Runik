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

pub mod app;
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
    let app = app::App::load_from_img();
    println!("[kernel] [trace] app entry point: {:#x}", app.get_entry_point());
    let (kernel_space, user_stack_base) = crate::mm::addr_space::AddrSpace::new_with_elf(&app.elf_file);
    println!("[kernel] [debug] user_sp: {:p}", user_stack_base as *const usize);
    kernel_space.activate();
    println!("[kernel] [trace] Paging mode activated");
    println!("[kernel] [info] Running user's application");
    app.run(user_stack_base);
    /*let kernel_space: Arc<UPIntrFreeCell<AddrSpace>> =
        Arc::new(unsafe { UPIntrFreeCell::new() });*/
}
