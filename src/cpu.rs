use std::fmt::{write, Display};

use crate::{
    gba_file::GBAHeader,
    instr::{
        arm::{Alu, AluOp, Branch, BranchExchange, Instruction, Sdt},
        common::{EResult, ExecErr, Register},
        thumb::{
            ThumbAddSub, ThumbAlu, ThumbAluOp, ThumbBranch, ThumbBranchOp, ThumbHiReg,
            ThumbHiRegOp, ThumbInstr, ThumbLongBranch, ThumbLsi, ThumbLsiOp, ThumbMcas,
            ThumbMcasOp, ThumbMls, ThumbMlsOp, ThumbMultLS, ThumbMultLSOp, ThumbPushPop,
            ThumbPushPopOp, ThumbRegShift, ThumbRegShiftOp,
        },
    },
    logging,
};

#[derive(Debug, Default)]
pub struct Cpu {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    /// R13
    pub sp: u32,
    /// R14
    pub lr: u32,
    /// R15
    pub pc: u32,
    /// N - Sign Flag (false(0)=Not Signed, true(1)=Signed)
    sign_flag: bool,
    /// Z - Zero Flag (false(0)=Not Zero, true(1)=Zero)
    zero_flag: bool,
    /// C - Carry Flag (false(0)=Borrow/Not Carry, true(1)=Carry/No Borrow)
    carry_flag: bool,
    /// V - Overflow Flag (false(0)=No Overflow, true(1)=Overflow)
    overflow_flag: bool,
    thumb: bool,

    logging: bool,
    memory: Vec<u8>,
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cpu {{")?;
        writeln!(f, "    r0: 0x{:08x},", self.r0)?;
        writeln!(f, "    r1: 0x{:08x},", self.r1)?;
        writeln!(f, "    r2: 0x{:08x},", self.r2)?;
        writeln!(f, "    r3: 0x{:08x},", self.r3)?;
        writeln!(f, "    r4: 0x{:08x},", self.r4)?;
        writeln!(f, "    r5: 0x{:08x},", self.r5)?;
        writeln!(f, "    r6: 0x{:08x},", self.r6)?;
        writeln!(f, "    r7: 0x{:08x},", self.r7)?;
        writeln!(f, "    r13/sp: 0x{:08x},", self.sp)?;
        writeln!(f, "    r14/lr: 0x{:08x},", self.lr)?;
        writeln!(f, "    r15/pc: 0x{:08x},", self.pc)?;
        writeln!(f, "    sign_flag: {},", self.sign_flag)?;
        writeln!(f, "    carry_flag: {},", self.carry_flag)?;
        writeln!(f, "    overflow_flag: {},", self.overflow_flag)?;
        writeln!(f, "    thumb: {},", self.thumb)?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            // TODO: actual memory mapping for smaller allocations
            memory: vec![0; 0x10000000],
            ..Default::default()
        }
    }

    pub fn set_logging(&mut self, logging: bool) {
        self.logging = logging;
    }

    fn get_register(&self, reg: Register) -> EResult<u32> {
        match reg {
            Register::R0 => Ok(self.r0),
            Register::R1 => Ok(self.r1),
            Register::R2 => Ok(self.r2),
            Register::R3 => Ok(self.r3),
            Register::R4 => Ok(self.r4),
            Register::R5 => Ok(self.r5),
            Register::R6 => Ok(self.r6),
            Register::R7 => Ok(self.r7),
            Register::R13 => Ok(self.sp),
            Register::R14 => Ok(self.lr),
            Register::R15 => Ok(self.pc),
            _ => Err(ExecErr::UnimplementedInstr(format!(
                "Register {reg:?} not implmented"
            ))),
        }
    }

    fn set_register(&mut self, reg: Register, value: u32) -> EResult<()> {
        match reg {
            Register::R0 => self.r0 = value,
            Register::R1 => self.r1 = value,
            Register::R2 => self.r2 = value,
            Register::R3 => self.r3 = value,
            Register::R4 => self.r4 = value,
            Register::R5 => self.r5 = value,
            Register::R6 => self.r6 = value,
            Register::R7 => self.r7 = value,
            Register::R13 => self.sp = value,
            Register::R14 => self.lr = value,
            Register::R15 => self.pc = value,
            _ => {
                return Err(ExecErr::UnimplementedInstr(format!(
                    "Register {reg:?} not implmented"
                )))
            }
        }

        Ok(())
    }

    pub fn get_memory(&self, offset: u32) -> u32 {
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
            let addr = match sdt.rn {
                Register::R15 => addr + 8,
                _ => addr,
            };
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
        logging!(self.logging, "{}", fmt);

        let instr: Instruction = word.try_into()?;

        let fmt = format!("Executing: {instr:?}");
        logging!(self.logging, "{}", fmt);

        match instr {
            Instruction::Branch(b) => self.run_branch(b)?,
            Instruction::BranchExchange(b) => self.run_branch_exhange(b)?,
            Instruction::Alu(a) => self.run_alu(a)?,
            Instruction::Sdt(sdt) => self.run_sdt(sdt)?,
            Instruction::Psr => {
                logging!(self.logging, "{}", "Ignoring Psr instructions");
                self.pc += 4;
            }
        }

        Ok(())
    }

    fn run_thumb_alu(&mut self, alu: ThumbAlu) -> EResult<()> {
        match alu.op {
            ThumbAluOp::Bic => {
                let not = !self.get_register(alu.rs)?;
                let value = self.get_register(alu.rd)? & not;
                self.set_register(alu.rd, value)?;
                self.zero_flag = self.get_register(alu.rd)? == 0;
            }
            ThumbAluOp::Cmp => {
                // TODO: other flags too
                self.zero_flag = self.get_register(alu.rd)? - self.get_register(alu.rs)? == 0;
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_lsi(&mut self, lsi: ThumbLsi) -> EResult<()> {
        match lsi.op {
            ThumbLsiOp::Str => {
                let base_addr = self.get_register(lsi.rb)?;
                let addr = base_addr + lsi.nn as u32;
                self.set_memory(addr, self.get_register(lsi.rd)?);
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_hireg(&mut self, hireg: ThumbHiReg) -> EResult<()> {
        match hireg.op {
            ThumbHiRegOp::Bx => {
                let destination = self.get_register(hireg.rd)?;
                // not completely sure why 1 is anded to lr/R14 in long jump
                // but now we have be sure it's removed
                self.pc = destination ^ 1;
                return Ok(());
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_mls(&mut self, mls: ThumbMls) -> EResult<()> {
        match mls.op {
            ThumbMlsOp::Ldr => {
                let mem_offset = self.get_register(mls.rb)? + mls.nn as u32;
                // take (PC and not 2) into account
                let mem_offset = mem_offset & !2;
                let value = self.get_memory(mem_offset);
                self.set_register(mls.rd, value)?
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_reg_shift(&mut self, reg_shift: ThumbRegShift) -> EResult<()> {
        let mut set_carry = true;
        match reg_shift.op {
            ThumbRegShiftOp::Lsl => {
                let value = self.get_register(reg_shift.rs)?;
                let value = (value & 0x80000000) | ((value & 0x7fffffff) << reg_shift.nn);
                self.set_register(reg_shift.rd, value)?;
                if reg_shift.nn == 0 {
                    set_carry = false;
                }
            }
            ThumbRegShiftOp::Lsr => todo!(),
            ThumbRegShiftOp::Asr => {
                let value = self.get_register(reg_shift.rs)?;
                let value = (value & 0x80000000) | ((value & 0x7fffffff) >> reg_shift.nn);
                self.set_register(reg_shift.rd, value)?;
                if reg_shift.nn == 0 {
                    set_carry = false;
                }
            }
        }

        self.zero_flag = true;
        // FIXME: set sign flag properly
        // self.sign_flag = true;
        if set_carry {
            self.carry_flag = true;
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_mcas(&mut self, mcas: ThumbMcas) -> EResult<()> {
        match mcas.op {
            ThumbMcasOp::Mov => {
                self.set_register(mcas.rd.clone(), mcas.nn as u32)?;
                self.zero_flag = self.get_register(mcas.rd)? == 0;
            }
            ThumbMcasOp::Sub => {
                /// TODO: handle underflow
                self.set_register(
                    mcas.rd.clone(),
                    self.get_register(mcas.rd)? - mcas.nn as u32,
                )?;
                self.zero_flag = self.get_register(mcas.rd)? == 0;
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_add_sub(&mut self, add_sub: ThumbAddSub) -> EResult<()> {
        match add_sub {
            ThumbAddSub::Addr(op) => {
                // TODO: handle overflows
                let value = self.get_register(op.rs)? + self.get_register(op.rn)?;
                self.set_register(op.rd, value)?;
            }
            ThumbAddSub::Subr(op) => {
                // TODO: handle undeflows
                let value = self.get_register(op.rs)? - self.get_register(op.rn)?;
                self.set_register(op.rd, value)?;
            }
            ThumbAddSub::Addi(op) => todo!(),
            ThumbAddSub::Subi(op) => todo!(),
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_push_pop(&mut self, push_pop: ThumbPushPop) -> EResult<()> {
        match push_pop.op {
            ThumbPushPopOp::Push => {
                for register in push_pop.rlist {
                    let memaddr = self.get_register(Register::R13)?;
                    let value = self.get_register(register)?;
                    self.set_memory(memaddr, value);
                    self.set_register(Register::R13, memaddr - 4)?;
                }
            }
            ThumbPushPopOp::Pop => {
                for register in push_pop.rlist {
                    let memaddr = self.get_register(Register::R13)?;
                    self.set_register(register, self.get_memory(memaddr))?;
                    self.set_register(Register::R13, memaddr + 4)?;
                }
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_multiple_load_store(&mut self, multls: ThumbMultLS) -> EResult<()> {
        match multls.op {
            ThumbMultLSOp::STMIA => {
                for register in multls.rlist {
                    let memaddr = self.get_register(multls.rb)?;
                    self.set_memory(memaddr, self.get_register(register)?);
                    self.set_register(multls.rb, memaddr + 4);
                }
            }
            ThumbMultLSOp::LDMIA => {
                for register in multls.rlist {
                    let memaddr = self.get_register(multls.rb)?;
                    self.set_register(register, self.get_memory(memaddr))?;
                    self.set_register(multls.rb, memaddr + 4);
                }
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_branch(&mut self, branch: ThumbBranch) -> EResult<()> {
        match branch.op {
            ThumbBranchOp::Beq => {
                if self.zero_flag {
                    // TODO: handle signed offsets
                    self.pc += (branch.offset * 2 + 4) as u32;
                    return Ok(());
                }
            }
            ThumbBranchOp::Bne => {
                if !self.zero_flag {
                    // NOTE: is it save to treat pc as i32?
                    self.pc = (self.pc as i32 + (branch.offset as i32 * 2 + 4)) as u32;
                    return Ok(());
                }
            }
            ThumbBranchOp::Bcs => {
                if self.carry_flag {
                    // TODO: handle signed offsets
                    self.pc += (branch.offset * 2 + 4) as u32;
                    return Ok(());
                }
            }
        }

        self.pc += 2;
        Ok(())
    }

    fn run_thumb_long_branch(&mut self, branch: ThumbLongBranch) -> EResult<()> {
        self.lr = (self.pc + 4) | 1;
        self.pc += 4 + branch.target;
        Ok(())
    }

    fn run_next_thumb_instr(&mut self) -> EResult<()> {
        let half_word = u16::from_le_bytes(
            self.memory[self.pc as usize..self.pc as usize + 2]
                .try_into()
                .unwrap(),
        );

        let fmt = format!(
            "Trying from half word: {half_word:04X} addr: {:08X}",
            self.pc
        );
        logging!(self.logging, "{}", fmt);

        let instr: EResult<ThumbInstr> = half_word.try_into();
        let instr = match instr {
            Ok(instr) => instr,
            Err(err) if err == ExecErr::LongInstruction => {
                let half_word2 = u16::from_le_bytes(
                    self.memory[self.pc as usize + 2..self.pc as usize + 4]
                        .try_into()
                        .unwrap(),
                );

                ThumbInstr::try_from_long(half_word, half_word2)?
            }
            Err(err) => return Err(err),
        };

        let fmt = format!("Executing: {instr:?}");
        logging!(self.logging, "{}", fmt);

        match instr {
            ThumbInstr::Alu(alu) => self.run_thumb_alu(alu)?,
            ThumbInstr::Lsi(lsi) => self.run_thumb_lsi(lsi)?,
            ThumbInstr::HiReg(hireg) => self.run_thumb_hireg(hireg)?,
            ThumbInstr::Mls(mls) => self.run_thumb_mls(mls)?,
            ThumbInstr::Mcas(mcas) => self.run_thumb_mcas(mcas)?,
            ThumbInstr::AddSub(add_sub) => self.run_add_sub(add_sub)?,
            ThumbInstr::MultLS(multls) => self.run_thumb_multiple_load_store(multls)?,
            ThumbInstr::PushPop(push_pop) => self.run_thumb_push_pop(push_pop)?,
            ThumbInstr::Branch(branch) => self.run_thumb_branch(branch)?,
            ThumbInstr::LongBranch(branch) => self.run_thumb_long_branch(branch)?,
            ThumbInstr::RegShift(reg_shift) => self.run_thumb_reg_shift(reg_shift)?,
        }

        Ok(())
    }

    pub fn initialize_cpu(&mut self, bytes: &[u8]) {
        let rom = GBAHeader::from_file(bytes);
        self.pc = 0x8000000;
        self.lr = 0x8000000;

        for (idx, b) in bytes.iter().enumerate() {
            self.memory[self.pc as usize + idx] = *b;
        }
    }

    pub fn execute_next(&mut self) -> EResult<()> {
        if self.thumb {
            self.run_next_thumb_instr()?;
        } else {
            self.run_next_instruction()?;
        }

        Ok(())
    }

    pub fn run_rom(&mut self, bytes: &[u8]) -> EResult<()> {
        self.initialize_cpu(bytes);

        loop {
            self.execute_next()?
        }
    }
}
