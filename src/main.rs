use std::fs;

use cpu::Cpu;

mod cpu;
mod gba_file;

fn main() {
    let bytes = fs::read("demos.gba").unwrap();

    let mut cpu = Cpu::new();
    let res = cpu.run_rom(&bytes);
    match res {
        Ok(_) => todo!(),
        Err(e) => match e {
            cpu::ExecErr::UnknownInstr(instr) => println!("Unknown instr {instr:08X}"),
            cpu::ExecErr::UnimplementedInstr(instr) => println!("Unimplemented '{instr}'",),
        },
    }
}
