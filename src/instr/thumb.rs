use super::common::{EResult, ExecErr, Register};

#[derive(Debug)]
pub enum ThumbAluOp {
    /// bit clear, Rd = Rd AND NOT Rs
    Bic,
}

#[derive(Debug)]
pub struct ThumbAlu {
    pub op: ThumbAluOp,
    /// Destination register
    pub rd: Register,
    /// Source register
    pub rs: Register,
}

impl TryFrom<u16> for ThumbAlu {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let op = match (value >> 6) & 0b1111 {
            0xe => ThumbAluOp::Bic,
            _ => unreachable!(),
        };

        let rs = Register::from((value >> 3) & 0b111);
        let rd = Register::from(value & 0b111);
        Ok(Self { op, rd, rs })
    }
}

#[derive(Debug)]
pub enum ThumbHiRegOp {
    /// BX  Rs ;jump PC = Rs ;may switch THUMB/ARM
    Bx,
}

/// THUMB.5: Hi register operations/branch exchange
#[derive(Debug)]
pub struct ThumbHiReg {
    pub op: ThumbHiRegOp,
    /// Destination register
    pub rd: Register,
}

impl TryFrom<u16> for ThumbHiReg {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let op = match (value >> 8) & 0b11 {
            3 => ThumbHiRegOp::Bx,
            _ => unreachable!(),
        };

        let rd = Register::from((value >> 3) & 0b1111);
        Ok(Self { op, rd })
    }
}

#[derive(Debug)]
pub enum ThumbMlsOp {
    Ldr,
}

/// Thumb mode memory load/store
#[derive(Debug)]
pub struct ThumbMls {
    pub op: ThumbMlsOp,
    /// Destination register
    pub rd: Register,
    /// Base register
    pub rb: Register,
    pub nn: u16,
}

#[derive(Debug)]
pub enum ThumbRegShiftOp {
    /// logical/arithmetic shift left
    Lsl,
    /// logical shift right
    Lsr,
    /// arithmetic shift right
    Asr,
}

#[derive(Debug)]
pub struct ThumbRegShift {
    pub op: ThumbRegShiftOp,
    /// Destination register
    pub rd: Register,
    /// Source register
    pub rs: Register,
    pub nn: u16,
}

impl TryFrom<u16> for ThumbRegShift {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let op = match (value >> 11) & 0b11 {
            0b00 => ThumbRegShiftOp::Lsl,
            0b01 => ThumbRegShiftOp::Lsr,
            0b10 => ThumbRegShiftOp::Asr,
            _ => unreachable!(),
        };

        let nn = (value >> 6) & 0b11111;
        let rs = Register::from((value >> 3) & 0b111);
        let rd = Register::from(value & 0b111);

        Ok(Self { op, nn, rs, rd })
    }
}

#[derive(Debug)]
pub enum ThumbBranchOp {
    /// BEQ label ;Z=1 ;equal (zero) (same)
    Beq,
    /// BNE label ;Z=0 ;not equal (nonzero) (not same)
    Bne,
    /// BCS/BHS label ;C=1 ;unsigned higher or same (carry set)
    Bcs,
}

/// THUMB.16: conditional branch and THUMB.18: unconditional branch
#[derive(Debug)]
pub struct ThumbBranch {
    pub op: ThumbBranchOp,
    /// Signed Offset, step 2 ($+4-256..$+4+254)
    pub offset: i16,
}

impl TryFrom<u16> for ThumbBranch {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let op = match (value >> 8) & 0b1111 {
            0x0 => ThumbBranchOp::Beq,
            0x1 => ThumbBranchOp::Bne,
            0x2 => ThumbBranchOp::Bcs,
            _ => unreachable!(),
        };

        // Hacky way to get the value as unsigned
        let offset = (value & 0xff) as i8 as i16;

        Ok(Self { op, offset })
    }
}

/// THUMB.19: long branch with link
/// Assumes that opcode is always BL, and BLX is not supported
#[derive(Debug)]
pub struct ThumbLongBranch {
    /// The destination address range (PC+4)-400000h..+3FFFFEh, ie. PC+/-4M.
    pub target: u32,
}

/// THUMB.2: add/subtract immediate
#[derive(Debug)]
pub struct ThumbAddSubI {
    /// Destination register
    pub rd: Register,
    /// Source register
    pub rs: Register,
    /// Immediate value
    pub nn: u16,
}

/// THUMB.2: add/subtract register
#[derive(Debug)]
pub struct ThumbAddSubR {
    /// Destination register
    pub rd: Register,
    /// Source register
    pub rs: Register,
    /// Operand register
    pub rn: Register,
}

/// THUMB.2: add/subtract
#[derive(Debug)]
pub enum ThumbAddSub {
    /// add register Rd=Rs+Rn
    Addr(ThumbAddSubR),
    /// subtract register Rd=Rs-Rn
    Subr(ThumbAddSubR),
    /// add immediate Rd=Rs+nn
    Addi(ThumbAddSubI),
    /// subtract immediate Rd=Rs-nn
    Subi(ThumbAddSubI),
}

impl TryFrom<u16> for ThumbAddSub {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let rs = Register::from((value >> 3) & 0b111);
        let rd = Register::from(value & 0b111);
        let nn = (value >> 6) & 0b111;
        let rn = Register::from(nn);
        let register = ThumbAddSubR { rd, rs, rn };
        let immediate = ThumbAddSubI { rd, rs, nn };

        let op = match (value >> 9) & 0b11 {
            0 => ThumbAddSub::Addr(register),
            1 => ThumbAddSub::Subr(register),
            2 => ThumbAddSub::Addi(immediate),
            3 => ThumbAddSub::Subi(immediate),
            _ => unreachable!(),
        };

        Ok(op)
    }
}

#[derive(Debug)]
pub enum ThumbMcasOp {
    /// move Rd = #nn
    Mov,
    /// Rd,#nn ;subtract Rd   = Rd - #nn
    Sub,
}

/// THUMB.3: move/compare/add/subtract immediate
#[derive(Debug)]
pub struct ThumbMcas {
    pub op: ThumbMcasOp,
    /// Destination register
    pub rd: Register,
    pub nn: u16,
}

impl TryFrom<u16> for ThumbMcas {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let op = match (value >> 11) & 0b11 {
            0b00 => ThumbMcasOp::Mov,
            0b11 => ThumbMcasOp::Sub,
            _ => unreachable!(),
        };

        let rd = Register::from((value >> 8) & 0b111);
        let nn = value & 0xff;

        Ok(Self { op, nn, rd })
    }
}

#[derive(Debug)]
pub enum ThumbMultLSOp {
    /// Rb!,{Rlist};store in memory, increments Rb
    STMIA,
    /// LDMIA Rb!,{Rlist} ;load from memory, increments Rb
    LDMIA,
}

/// THUMB.15: multiple load/store
#[derive(Debug)]
pub struct ThumbMultLS {
    pub op: ThumbMultLSOp,
    /// Base register
    pub rb: Register,
    /// Register list
    /// in order of: R0 first, R1 second ... R7 last
    pub rlist: Vec<Register>,
}

impl TryFrom<u16> for ThumbMultLS {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let op = match (value >> 11) & 0b1 {
            0 => ThumbMultLSOp::STMIA,
            1 => ThumbMultLSOp::LDMIA,
            _ => unreachable!(),
        };

        let rb = Register::from((value >> 8) & 0b111);

        let mut list = value & 0xff;
        let mut rlist = Vec::new();

        for idx in (0..=7) {
            if list & 1 == 1 {
                rlist.push(Register::from(idx as u32));
            }
            list >>= 1;
        }

        Ok(Self { op, rb, rlist })
    }
}

#[derive(Debug)]
pub enum ThumbInstr {
    /// Memory load/store
    Mls(ThumbMls),
    /// THUMB.4: ALU operations
    Alu(ThumbAlu),
    /// THUMB.5: Hi register operations/branch exchange
    HiReg(ThumbHiReg),
    /// THUMB.3: move/compare/add/subtract immediate
    Mcas(ThumbMcas),
    /// THUMB.2: add/subtract
    AddSub(ThumbAddSub),
    /// THUMB.15: multiple load/store
    MultLS(ThumbMultLS),
    /// (Conditional) Branch
    Branch(ThumbBranch),
    /// THUMB.19: long branch with link
    LongBranch(ThumbLongBranch),
    /// THUMB.1: move shifted register
    RegShift(ThumbRegShift),
}

impl TryFrom<u16> for ThumbInstr {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        // THUMB.6: load PC-relative (for loading immediates from literal pool)
        if (value >> 11) & 0b11111 == 0b01001 {
            let rd = Register::from((value >> 8) & 0b111);
            let rb = Register::R15;
            // + 4 since PC register is evaluated as PC+4
            let nn = (value & 0xFF) * 4 + 4;

            Ok(ThumbInstr::Mls(ThumbMls {
                op: ThumbMlsOp::Ldr,
                rd,
                rb,
                nn,
            }))
        } else if (value >> 10) & 0b111111 == 0b010000 {
            Ok(ThumbInstr::Alu(ThumbAlu::try_from(value)?))
        } else if (value >> 10) & 0b111111 == 0b010001 {
            Ok(ThumbInstr::HiReg(ThumbHiReg::try_from(value)?))
        } else if (value >> 11) & 0b11111 == 0b00011 {
            Ok(ThumbInstr::AddSub(ThumbAddSub::try_from(value)?))
        } else if (value >> 13) & 0b111 == 0b000 {
            Ok(ThumbInstr::RegShift(ThumbRegShift::try_from(value)?))
        } else if (value >> 13) & 0b111 == 0b001 {
            Ok(ThumbInstr::Mcas(ThumbMcas::try_from(value)?))
        } else if (value >> 12) & 0b1111 == 0b1101 {
            Ok(ThumbInstr::Branch(ThumbBranch::try_from(value)?))
        } else if (value >> 12) & 0b1111 == 0b1100 {
            Ok(ThumbInstr::MultLS(ThumbMultLS::try_from(value)?))
        } else if (value >> 11) & 0b11111 == 0b11110 {
            Err(ExecErr::LongInstruction)
        } else {
            Err(ExecErr::UnknownThumbInstr(value))
        }
    }
}

impl ThumbInstr {
    pub fn try_from_long(instr1: u16, instr2: u16) -> EResult<Self> {
        // long branch with BL op code
        if (instr1 >> 11) & 0b11111 == 0b11110 && (instr2 >> 11) & 0b11111 == 0b11111 {
            let target = ((instr1 as u32) & 0x7ff) << 12 | ((instr2 as u32) & 0x7ff) << 1;
            /*    println!("found {:08X} {:08X}", instr1, (instr1 as u32) & 0x7ff);
            println!("found {:08X} {:08X}", instr2, (instr2 as u32) & 0x7ff);
            panic!("found {target:08X}"); */
            Ok(ThumbInstr::LongBranch(ThumbLongBranch { target }))
        } else {
            // TODO: own error for long thum instr
            Err(ExecErr::UnknownInstr(instr1 as u32 | (instr2 as u32) << 16))
        }
    }
}
