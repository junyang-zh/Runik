use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::env;

static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn main() {
    let platform = env::var("RUNIK_PLATFORM").unwrap();
    let linker_script_path: &str;
    if platform == "qemu" {
        linker_script_path = "./src/plat/qemu/linker.ld";
    } else {
        panic!("Platform {} not supported!", platform);
    }
    println!("cargo:rustc-link-arg=-T{}", linker_script_path);
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().unwrap();
}

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let app = &read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect::<Vec<_>>()[0];

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
    .incbin "{1}{0}.bin"
app_end:
"#,
        app, TARGET_PATH
    )?;

    Ok(())
}
