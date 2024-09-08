use std::fs;

use cpu::Cpu;
use instr::common::ExecErr;

mod cpu;
mod gba_file;
mod instr;

fn main() {
    let bytes = fs::read("demos.gba").unwrap();

    let mut cpu = Cpu::new();
    let res = cpu.run_rom(&bytes);
    match res {
        Ok(_) => todo!(),
        Err(e) => match e {
            ExecErr::UnknownInstr(instr) => println!("Unknown instr {instr:08X}"),
            ExecErr::UnimplementedInstr(instr) => println!("Unimplemented '{instr}'",),
        },
    }
}
