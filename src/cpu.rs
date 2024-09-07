use crate::gba_file::GBAHeader;

pub enum ExecErr {
    UnknownInstr(u32),
    UnimplementedInstr(String),
}

pub type EResult<T> = Result<T, ExecErr>;

#[derive(Debug)]
enum Condition {
    Eq,
    Ne,
    Cs,
    Cc,
    Mi,
    Pl,
    Vs,
    Vc,
    Hi,
    Ls,
    Ge,
    Lt,
    Gt,
    Le,
    Al,
    Nv,
}

impl From<u32> for Condition {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Eq,
            1 => Self::Ne,
            2 => Self::Cs,
            3 => Self::Cc,
            4 => Self::Mi,
            5 => Self::Pl,
            6 => Self::Vs,
            7 => Self::Vc,
            8 => Self::Hi,
            9 => Self::Ls,
            10 => Self::Ge,
            11 => Self::Lt,
            12 => Self::Gt,
            13 => Self::Le,
            14 => Self::Al,
            15 => Self::Nv,
            _ => unreachable!("Unkown condition {value:x}"),
        }
    }
}

#[derive(Debug)]
struct Branch {
    condition: Condition,
    /// is B or BL, true for Bl
    is_link: bool,
    nn: u32,
}

impl From<u32> for Branch {
    fn from(value: u32) -> Self {
        let condition = Condition::from((value >> 28) & 0b1111);
        let opcode = (value >> 24) & 0b1;
        let nn = (value) & 0xffffff;
        Self {
            condition,
            is_link: opcode == 1,
            nn,
        }
    }
}

#[derive(Debug)]
enum AluOp {
    And,
    Eor,
    Sub,
    Rsb,
    Add,
    Adc,
    Sbc,
    Rsc,
    Tst,
    Teq,
    Cmp,
    Cmn,
    Orr,
    Mov,
    Bic,
    Mvn,
}

impl From<u32> for AluOp {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::And,
            1 => Self::Eor,
            2 => Self::Sub,
            3 => Self::Rsb,
            4 => Self::Add,
            5 => Self::Adc,
            6 => Self::Sbc,
            7 => Self::Rsc,
            8 => Self::Tst,
            9 => Self::Teq,
            10 => Self::Cmp,
            11 => Self::Cmn,
            12 => Self::Orr,
            13 => Self::Mov,
            14 => Self::Bic,
            15 => Self::Mvn,
            _ => unreachable!("Unknown AluOp {value:x}"),
        }
    }
}

#[derive(Debug)]
struct Alu {
    condition: Condition,
    immediate: bool,
    op: AluOp,
    /// Data that's executed based in immediate mode
    operand: u32,
    s: bool,
    rn: Register,
    rd: Register,
}

impl From<u32> for Alu {
    fn from(value: u32) -> Self {
        let condition = Condition::from((value >> 28) & 0b1111);
        let immediate = (value >> 25) & 0b1 == 1;
        let op = AluOp::from((value >> 21) & 0b1111);
        let s = (value >> 20) & 0b1 == 1;
        let rn = Register::from((value >> 16) & 0b1111);
        let rd = Register::from((value >> 12) & 0b1111);
        let operand = value & 0xffffff;

        Self {
            condition,
            immediate,
            op,
            s,
            rn,
            rd,
            operand,
        }
    }
}

#[derive(Debug)]
struct Sdt {
    condition: Condition,
    immediate: bool,
    pre: bool,
    up: bool,
    bit: bool,
    tw: bool,
    load_memory: bool,
    rn: Register,
    rd: Register,
    /// Data that's executed based in immediate mode
    operand: u32,
}

impl From<u32> for Sdt {
    fn from(value: u32) -> Self {
        let condition = Condition::from((value >> 28) & 0b1111);
        let immediate = (value >> 25) & 0b1 == 0;
        let pre = (value >> 24) & 0b1 == 1;
        let up = (value >> 23) & 0b1 == 1;
        let bit = (value >> 22) & 0b1 == 1;
        let tw = (value >> 21) & 0b1 == 1;
        let load_memory = (value >> 20) & 0b1 == 1;
        let rn = Register::from((value >> 16) & 0b1111);
        let rd = Register::from((value >> 12) & 0b1111);
        let operand = value & 0xfff;

        Self {
            condition,
            immediate,
            pre,
            up,
            bit,
            tw,
            load_memory,
            rn,
            rd,
            operand,
        }
    }
}

#[derive(Debug)]
enum Instruction {
    Branch(Branch),
    Alu(Alu),
    /// Single Data Tranfer, LDR, STR, PLD
    Sdt(Sdt),
    /// PSR Transfer (MRS, MSR)
    Psr,
}

impl TryFrom<u32> for Instruction {
    type Error = ExecErr;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (value >> 25) & 0b111 == 0b101 {
            Ok(Self::Branch(Branch::from(value)))
        } else if (value >> 26) & 0b11 == 0b00 {
            let op = AluOp::from((value >> 21) & 0b1111);
            let s = (value >> 20) & 0b1;
            if s == 0 && matches!(op, AluOp::Tst | AluOp::Teq | AluOp::Cmp | AluOp::Cmn) {
                return Ok(Instruction::Psr);
            }

            Ok(Self::Alu(Alu::from(value)))
        } else if (value >> 26) & 0b01 == 0b01 {
            Ok(Self::Sdt(Sdt::from(value)))
        } else {
            Err(ExecErr::UnknownInstr(value))
        }
    }
}

#[derive(Debug, PartialEq)]
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    /// (SP)
    R13,
    /// (LR)
    R14,
    /// (PC)
    R15,
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::R0,
            1 => Self::R1,
            2 => Self::R2,
            3 => Self::R3,
            4 => Self::R4,
            5 => Self::R5,
            6 => Self::R6,
            7 => Self::R7,
            8 => Self::R8,
            9 => Self::R9,
            10 => Self::R10,
            11 => Self::R11,
            12 => Self::R12,
            13 => Self::R13,
            14 => Self::R14,
            15 => Self::R15,
            _ => unreachable!("Unknown register {value:x}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Cpu {
    pub r0: u32,
    /// R13
    pub sp: u32,
    /// R15
    pub pc: u32,
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

        let fmt = format!("Trying from word: {word:08X} addr: {:08X}", self.pc);
        dbg!(fmt);

        let instr: Instruction = word.try_into()?;

        let fmt = format!("Executing: {instr:#?}");
        dbg!(fmt);

        match instr {
            Instruction::Branch(b) => self.run_branch(b)?,
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
