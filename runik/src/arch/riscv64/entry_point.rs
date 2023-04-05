//! The entry point of runik

use core::arch::global_asm;

global_asm!(include_str!("entry_point.asm"));
