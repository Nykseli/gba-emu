use std::fs;

use cpu::Cpu;

mod cpu;
mod gba_file;

fn main() {
    let bytes = fs::read("demos.gba").unwrap();

    let mut cpu = Cpu::new();
    cpu.run_rom(&bytes);
}
