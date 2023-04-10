//! Kernel stack
use crate::arch::trap::TrapContext;
use crate::plat::qemu::MEMORY_END;
use crate::arch::paging::PAGE_SIZE;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position() -> (usize, usize) {
    let top = MEMORY_END - (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

pub fn push_context(cx: TrapContext) -> &'static mut TrapContext {
    let cx_ptr = (kernel_stack_position().1 - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
    unsafe {
        *cx_ptr = cx;
    }
    unsafe { cx_ptr.as_mut().unwrap() }
}
