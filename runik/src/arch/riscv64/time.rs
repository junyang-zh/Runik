//! the machine clock and time module

use riscv::register::time;

pub fn get_clock() -> usize {
    time::read()
}
