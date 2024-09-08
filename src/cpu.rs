use crate::{
    gba_file::GBAHeader,
    instr::{
        arm::{Alu, AluOp, Branch, BranchExchange, Instruction, Sdt},
        common::{EResult, ExecErr, Register},
    },
};

#[derive(Debug, Default)]
pub struct Cpu {
    pub r0: u32,
    /// R13
    pub sp: u32,
    /// R15
    pub pc: u32,
    thumb: bool,
    memory: Vec<u8>,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            // TODO: actual memory mapping for smaller allocations
            memory: vec![0; 0x10000000],
            ..Default::default()
        }
    }

    fn get_register(&self, reg: Register) -> EResult<u32> {
        match reg {
            Register::R0 => Ok(self.r0),
            Register::R13 => Ok(self.sp),
            Register::R15 => Ok(self.pc),
            _ => Err(ExecErr::UnimplementedInstr(format!(
                "Register {reg:?} not implmented"
            ))),
        }
    }

    fn set_register(&mut self, reg: Register, value: u32) -> EResult<()> {
        match reg {
            Register::R0 => self.r0 = value,
            Register::R13 => self.sp = value,
            Register::R15 => self.pc = value,
            _ => {
                return Err(ExecErr::UnimplementedInstr(format!(
                    "Register {reg:?} not implmented"
                )))
            }
        }

        Ok(())
    }

    fn get_memory(&self, offset: u32) -> u32 {
        u32::from_le_bytes(
            self.memory[offset as usize..offset as usize + 4]
                .try_into()
                .unwrap(),
        )
    }

    fn set_memory(&mut self, offset: u32, value: u32) {
        let bytes = value.to_le_bytes();
        self.memory[offset as usize] = bytes[0];
        self.memory[offset as usize + 1] = bytes[1];
        self.memory[offset as usize + 2] = bytes[2];
        self.memory[offset as usize + 3] = bytes[3];
    }

    fn run_branch(&mut self, branch: Branch) -> EResult<()> {
        // TODO: properly handle nn being signed
        // TODO: handle BL
        if branch.is_link {
            return Err(ExecErr::UnimplementedInstr(
                "Runnin BL is not implemented".into(),
            ));
        }

        self.pc = self.pc + 8 + branch.nn * 4;
        Ok(())
    }

    fn run_branch_exhange(&mut self, branch: BranchExchange) -> EResult<()> {
        let reg_value = self.get_register(branch.rn)?;
        let target = (reg_value | 1) - 1;
        self.pc = target;
        self.thumb = true;
        Ok(())
    }

    fn run_alu(&mut self, alu: Alu) -> EResult<()> {
        // TODO: condition codes
        match alu.op {
            AluOp::And => Err(ExecErr::UnimplementedInstr(
                "AluOp::And not implemented".into(),
            )),
            AluOp::Eor => Err(ExecErr::UnimplementedInstr(
                "AluOp::Eor not implemented".into(),
            )),
            AluOp::Sub => Err(ExecErr::UnimplementedInstr(
                "AluOp::Sub not implemented".into(),
            )),
            AluOp::Rsb => Err(ExecErr::UnimplementedInstr(
                "AluOp::Rsb not implemented".into(),
            )),
            AluOp::Add => {
                // TODO: move register
                if !alu.immediate {
                    return Err(ExecErr::UnimplementedInstr(
                        "AluOp::Add register value not supported".into(),
                    ));
                }

                let rors = (alu.operand >> 8) & 0b1111;
                let nn = alu.operand & 0b11111111;
                let op2 = nn.rotate_right(rors * 2);
                // When using R15 as operand (Rm or Rn), the returned value depends
                // on the instruction: PC+12 if I=0,R=1 (shift by register),
                // otherwise PC+8 (shift by immediate).
                let mut reg = if alu.rn == Register::R15 { 8 } else { 0 };

                reg += self.get_register(alu.rn)?;
                self.set_register(alu.rd, reg + op2)?;
                self.pc += 4;
                Ok(())
            }
            AluOp::Adc => Err(ExecErr::UnimplementedInstr(
                "AluOp::Adc not implemented".into(),
            )),
            AluOp::Sbc => Err(ExecErr::UnimplementedInstr(
                "AluOp::Sbc not implemented".into(),
            )),
            AluOp::Rsc => Err(ExecErr::UnimplementedInstr(
                "AluOp::Rsc not implemented".into(),
            )),
            AluOp::Tst => Err(ExecErr::UnimplementedInstr(
                "AluOp::Tst not implemented".into(),
            )),
            AluOp::Teq => Err(ExecErr::UnimplementedInstr(
                "AluOp::Teq not implemented".into(),
            )),
            AluOp::Cmp => Err(ExecErr::UnimplementedInstr(
                "AluOp::Cmp not implemented".into(),
            )),
            AluOp::Cmn => Err(ExecErr::UnimplementedInstr(
                "AluOp::Cmn not implemented".into(),
            )),
            AluOp::Orr => Err(ExecErr::UnimplementedInstr(
                "AluOp::Orr not implemented".into(),
            )),
            AluOp::Mov => {
                // TODO: move register
                if !alu.immediate {
                    return Err(ExecErr::UnimplementedInstr(
                        "AluOp::Mov register value not supported".into(),
                    ));
                }

                let rors = (alu.operand >> 8) & 0b1111;
                let nn = alu.operand & 0b11111111;
                self.set_register(alu.rd, nn.rotate_right(rors * 2))?;
                self.pc += 4;
                Ok(())
            }
            AluOp::Bic => Err(ExecErr::UnimplementedInstr(
                "AluOp::Bic not implemented".into(),
            )),
            AluOp::Mvn => Err(ExecErr::UnimplementedInstr(
                "AluOp::Mvn not implemented".into(),
            )),
        }
    }

    fn run_sdt(&mut self, sdt: Sdt) -> EResult<()> {
        // TODO: properly handle condition
        // TODO: properly handle tw (bit 21)

        if !sdt.immediate {
            return Err(ExecErr::UnimplementedInstr(
                "Runnin SDT register offset not implemented".into(),
            ));
        }

        if sdt.load_memory {
            let base_addr = self.get_register(sdt.rn)?;
            // TODO: proper unsigned addition
            let addr = base_addr + sdt.operand;
            self.set_register(sdt.rd, self.get_memory(addr))?;
        } else {
            self.set_memory(self.get_register(sdt.rn)? + sdt.operand, self.r0);
        }

        self.pc += 4;

        Ok(())
    }

    fn run_next_instruction(&mut self) -> EResult<()> {
        let word = u32::from_le_bytes(
            self.memory[self.pc as usize..self.pc as usize + 4]
                .try_into()
                .unwrap(),
        );

        if self.thumb {
            unimplemented!("Cannot run in thumb mode");
        }

        let fmt = format!("Trying from word: {word:08X} addr: {:08X}", self.pc);
        dbg!(fmt);

        let instr: Instruction = word.try_into()?;

        let fmt = format!("Executing: {instr:#?}");
        dbg!(fmt);

        match instr {
            Instruction::Branch(b) => self.run_branch(b)?,
            Instruction::BranchExchange(b) => self.run_branch_exhange(b)?,
            Instruction::Alu(a) => self.run_alu(a)?,
            Instruction::Sdt(sdt) => self.run_sdt(sdt)?,
            Instruction::Psr => {
                dbg!("Ignoring Psr instructions");
                self.pc += 4;
            }
        }

        Ok(())
    }

    pub fn run_rom(&mut self, bytes: &[u8]) -> EResult<()> {
        let rom = GBAHeader::from_file(bytes);
        self.pc = 0x8000000;

        for (idx, b) in bytes.iter().enumerate() {
            self.memory[self.pc as usize + idx] = *b;
        }

        loop {
            self.run_next_instruction()?
        }
    }
}
