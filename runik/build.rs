use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::env;
use std::path::PathBuf;
use std::vec::Vec;

use binsa::elf_syscalls;

// static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";
static TARGET_PATH: &str = "../app/target/";

fn main() {
    let platform = env::var("RUNIK_PLATFORM").unwrap();
    let linker_script_path: &str;
    if platform == "qemu" {
        linker_script_path = "./src/plat/qemu/linker.ld";
    } else {
        panic!("Platform {} not supported!", platform);
    }
    println!("cargo:rustc-link-arg=-T{}", linker_script_path);
    println!("cargo:rerun-if-changed={}/*", TARGET_PATH);
    insert_app_data().unwrap();
    let app_path = PathBuf::from(&(TARGET_PATH.to_owned() + &app_name_in_dir(TARGET_PATH).unwrap()));
    let app_abs_path = app_path.canonicalize().unwrap();
    let app_abs_path_str = app_abs_path.as_os_str().to_str().unwrap();
    let syscalls = elf_syscalls(&app_abs_path_str);
    // panic!("{:?}", syscalls);
    for id in syscalls {
        println!("cargo:rustc-cfg=syscall{}", id);
    }
}

fn app_name_in_dir(path: &str) -> Option<String> {
    read_dir(path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            dir_entry.unwrap().file_name().into_string().unwrap()
        })
        .collect::<Vec<_>>().first().cloned()
}

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let app = &app_name_in_dir(TARGET_PATH).unwrap();
    writeln!(
        f,
r#"
    .align 3
    .section .data
    .global app_start_addr
    .global app_end_addr
app_start_addr:
    .quad app_start
app_end_addr:
    .quad app_end
app_start:
    .incbin "{0}{1}"
app_end:
"#,
        TARGET_PATH, app
    )?;

    Ok(())
}
