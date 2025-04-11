use std::{
    io::{self, BufRead, Write},
    process::exit,
};

use crate::{cpu::Cpu, instr::common::EResult};

pub struct Debugger {
    cpu: Cpu,
    breaks: Vec<u32>,
}

impl Debugger {
    pub fn new(cpu: Cpu) -> Self {
        Self {
            cpu,
            breaks: Vec::new(),
        }
    }

    pub fn initialize(&mut self, bytes: &[u8]) {
        self.cpu.initialize_cpu(bytes);
    }

    fn run(&mut self) -> EResult<()> {
        loop {
            if self.breaks.contains(&self.cpu.pc) {
                println!("break on addr {:08x}", self.cpu.pc);
                break;
            }
            self.cpu.execute_next()?;
        }

        Ok(())
    }

    fn add_break(&mut self, cmd: &str) {
        let addr = cmd.split_whitespace().nth(1).unwrap();
        let addr = u32::from_str_radix(addr, 16).unwrap();
        self.breaks.push(addr);
    }

    fn print_value(&mut self, cmd: &str) {
        let addr = cmd.split_whitespace().nth(1).unwrap();
        let addr = u32::from_str_radix(addr, 16).unwrap();
        let value = self.cpu.get_memory(addr);
        println!("value found {:08x}", value);
    }

    fn run_command(&mut self, cmd: &str) -> EResult<()> {
        if cmd == "q" || cmd == "quit" || cmd == "exit" {
            exit(0);
        } else if cmd == "p" || cmd == "print" {
            println!("{}", self.cpu);
        } else if cmd == "r" || cmd == "run" {
            self.run()?;
        } else if cmd == "n" || cmd == "next" {
            self.cpu.execute_next()?;
        } else if cmd.starts_with("v ") || cmd.starts_with("value ") {
            self.print_value(cmd);
        } else if cmd.starts_with("b ") || cmd.starts_with("break ") {
            self.add_break(cmd);
        } else {
            println!("Unknown command {cmd}");
        }

        Ok(())
    }

    pub fn run_file(&mut self, file_data: &str) -> EResult<()> {
        for line in file_data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('#') {
                continue;
            }

            self.run_command(line)?;
        }

        Ok(())
    }

    pub fn repl(&mut self) -> EResult<()> {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut cmd = String::new();
            let _ = io::stdin().lock().read_line(&mut cmd).unwrap();
            self.run_command(&cmd.trim())?;
        }

        Ok(())
    }
}
