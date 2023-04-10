//! Loader for binary apps

use crate::sync::UPSafeCell;
//use core::arch::asm;
use core::slice;
use core::mem;
use lazy_static::*;

// use crate::mm::addr_space::AddrSpace;

pub struct AppManager {
    pub entry_point: usize,
    pub user_stack_base: usize,
    app_memrange: [usize; 2],
}

impl AppManager {
    pub fn print_app_info(&self) {
        println!(
            "[kernel] app memrange: [{:#x}, {:#x})",
            self.app_memrange[0],
            self.app_memrange[1]
        );
    }

    pub unsafe fn load_app(&self) -> &[u8] {
        println!("[kernel] Loading app");
        // clear app area
        // core::slice::from_raw_parts_mut(self.entry_point as *mut u8, APP_SIZE_LIMIT).fill(0);
        let start: usize = self.app_memrange[0];
        let end: usize = self.app_memrange[1];
        /*let app_src = core::slice::from_raw_parts(
            start as *const u8,
            end - start,
        );*/
        let buf: &[u8] = slice::from_raw_parts(start as *const u8, (end - start) / mem::size_of::<u8>());

        
        //let app_dst = core::slice::from_raw_parts_mut(entry_point as *mut u8, app_src.len());
        //app_dst.copy_from_slice(app_src);
        //asm!("fence.i");
        buf
    }
}

lazy_static! {
    pub static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn app_start_addr();
                fn app_end_addr();
            }
            // Read u64 since we stored them in .quad values
            let start: usize = (app_start_addr as u64 as *const u64).read_volatile().try_into().unwrap();
            let end: usize = (app_end_addr as u64 as *const u64).read_volatile().try_into().unwrap();
            AppManager {
                entry_point: 0,
                user_stack_base: 0,
                app_memrange: [start, end],
            }
        })
    };
}

/// init batch subsystem
pub fn init() {
    print_app_info();
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}
