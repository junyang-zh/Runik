//! Memory management

pub mod addr;
pub mod addr_space;
pub mod page_table;
pub mod frame;
pub mod kernel_heap;

pub fn init() {
    kernel_heap::init_heap();
    frame::init_frame_allocator();
    // addr_space::KERNEL_SPACE.exclusive_access().activate();
}
