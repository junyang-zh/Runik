//! static analysis of binary elf files

use std::vec::Vec;
use std::collections::HashSet;

use riscv_decode::{
    decode, instruction_length,
    Instruction::{ Addi, Lui, Ecall }
};

pub fn disasm_syscalls(text: &[u8]) -> Vec<usize> {
    let mut syscalls = HashSet::<usize>::new();
    let mut last_a7: usize = 0;
    let mut cur: usize = 0;
    let mut chunk = [0usize; 4];
    while cur < text.len() {
        (chunk[0], chunk[1]) = (text[cur] as usize, text[cur + 1] as usize);
        cur += 2;
        let decoded;
        if instruction_length((chunk[0] | (chunk[1] << 8)).try_into().unwrap()) == 4 {
            (chunk[2], chunk[3]) = (text[cur] as usize, text[cur + 1] as usize);
            cur += 2;
            decoded = decode((chunk[0] | (chunk[1] << 8) | (chunk[2] << 16) | (chunk[3] << 24)).try_into().unwrap());
            // print!("{:#x}", chunk[0] | (chunk[1] << 8) | (chunk[2] << 16) | (chunk[3] << 24));
        }
        else {
            decoded = decode((chunk[0] | (chunk[1] << 8)).try_into().unwrap());
            // print!("{:#x}", chunk[0] | (chunk[1] << 8));
        }
        match decoded {
            Ok(Lui(instr)) => {
                if instr.rd() == 17 {
                    last_a7 = instr.imm() as usize;
                    // print!(" lui, a7, {}", instr.imm());
                }
            },
            Ok(Addi(instr)) => {
                if instr.rd() == 17 {
                    if instr.rs1() == 0 {
                        last_a7 = instr.imm() as usize;
                        // print!(" addi, a7, x0, {}", instr.imm());
                    }
                    else if instr.rs1() == 17 {
                        last_a7 += instr.imm() as usize;
                        // print!(" addi, a7, a7, {}", instr.imm());
                    }
                }
            },
            Ok(Ecall) => {
                syscalls.insert(last_a7);
                // print!(" ecall");
            },
            Err(_e) => {
                // println!("Decoding error: {:?}", e);
            },
            _ => (),
        }
        // print!("\n");
    }
    syscalls.into_iter().collect()
}
