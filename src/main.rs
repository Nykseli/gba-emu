use std::{
    env::args,
    fs::{self, read_to_string},
};

use cpu::Cpu;
use debugger::Debugger;
use instr::common::ExecErr;
use video::Video;

mod cpu;
mod debugger;
mod gba_file;
mod instr;
mod logger;
mod video;

fn main() {
    let args: Vec<String> = args().collect();
    let debug = args.len() > 1 && (args[1] == "d" || args[1] == "debug");
    let bytes = fs::read("demos.gba").unwrap();

    let mut cpu = Cpu::new();

    let (res, cpu) = if debug {
        let mut debugger = Debugger::new(cpu);
        debugger.initialize(&bytes);
        if args.len() > 2 {
            let data = read_to_string(&args[2]).unwrap();
            (debugger.run_file(&data), debugger.cpu)
        } else {
            (debugger.repl(), debugger.cpu)
        }
    } else {
        (cpu.run_rom(&bytes, true), cpu)
    };

    match res {
        Ok(_) => {}
        Err(e) => match e {
            ExecErr::UnknownInstr(instr) => println!("Unknown instr {instr:08X}"),
            ExecErr::UnknownThumbInstr(instr) => println!("Unknown instr {instr:04X}"),
            ExecErr::UnimplementedInstr(instr) => println!("Unimplemented '{instr}'",),
            ExecErr::LongInstruction => {
                println!("Unexpected state where instruction needs more bytes to execute")
            }
        },
    }

    println!("{cpu}");

    let video = Video::new(cpu);
    video.initialize_screen();
    video.draw();
}
