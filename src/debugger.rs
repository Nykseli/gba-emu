use std::{
    io::{self, BufRead, Write},
    process::exit,
};

use crate::{cpu::Cpu, instr::common::EResult};

pub struct Debugger {
    pub cpu: Cpu,
    on_break: bool,
    breaks: Vec<u32>,
}

impl Debugger {
    pub fn new(cpu: Cpu) -> Self {
        Self {
            cpu,
            on_break: false,
            breaks: Vec::new(),
        }
    }

    pub fn initialize(&mut self, bytes: &[u8]) {
        self.cpu.initialize_cpu(bytes);
    }

    fn run(&mut self) -> EResult<()> {
        loop {
            if !self.on_break && self.breaks.contains(&self.cpu.pc) {
                println!("break on addr {:08x}", self.cpu.pc);
                self.on_break = true;
                break;
            }

            self.on_break = false;
            self.cpu.execute_next()?;
        }

        Ok(())
    }

    fn add_break(&mut self, cmd: &str) {
        let addr = cmd.split_whitespace().nth(1).unwrap();
        let addr = u32::from_str_radix(addr, 16).unwrap();
        self.breaks.push(addr);
    }

    fn add_relative_break(&mut self, cmd: &str) {
        let addr = cmd.split_whitespace().nth(1).unwrap();
        let addr = 0x08000000 | u32::from_str_radix(addr, 16).unwrap();
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
            self.on_break = false;
            self.cpu.execute_next()?
        } else if cmd == "logon" {
            self.cpu.set_logging(true);
        } else if cmd == "logoff" {
            self.cpu.set_logging(false);
        } else if cmd.starts_with("v ") || cmd.starts_with("value ") {
            self.print_value(cmd);
        } else if cmd.starts_with("b ") || cmd.starts_with("break ") {
            self.add_break(cmd);
        } else if cmd.starts_with("rb ") || cmd.starts_with("rbreak ") {
            self.add_relative_break(cmd);
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
            self.run_command(cmd.trim())?;
        }
    }
}
