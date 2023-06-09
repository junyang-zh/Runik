//! The module containing architecture specific implementations.
//! It is written using the crate `cfg_if` to do conditional compilation.

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        compile_error!("Arch `x86_64` not supported yet");
    } else if #[cfg(target_arch = "riscv64")] {
        #[path = "riscv64/trap/mod.rs"]
        pub mod trap;
        #[path = "riscv64/entry_point.rs"]
        pub mod entry_point;
        #[path = "riscv64/paging.rs"]
        pub mod paging;
        #[path = "riscv64/time.rs"]
        pub mod time;
        #[path = "riscv64/syscall.rs"]
        pub mod syscall;
    } else if #[cfg(target_arch = "aarch64")] {
        compile_error!("Arch `aarch64` not supported yet");
    } else {
        compile_error!("Invalid `target_arch` in cfg-if conditions");
    }
}
