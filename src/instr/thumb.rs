use super::common::{ExecErr, Register};

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
pub enum ThumbAluOp {
    /// logical/arithmetic shift left
    Lsl,
    /// logical shift right
    Lsr,
    /// arithmetic shift right
    Asr,
}

#[derive(Debug)]
pub struct ThumbAlu {
    pub op: ThumbAluOp,
    /// Destination register
    pub rd: Register,
    /// Source register
    pub rs: Register,
    pub nn: u16,
}

impl TryFrom<u16> for ThumbAlu {
    type Error = ExecErr;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let op = match (value >> 11) & 0b11 {
            0b00 => ThumbAluOp::Lsl,
            0b01 => ThumbAluOp::Lsr,
            0b10 => ThumbAluOp::Asr,
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
            0x2 => ThumbBranchOp::Bcs,
            _ => unreachable!(),
        };

        // TODO: check for unsigned
        let offset = (value & 0xff) as i16;

        Ok(Self { op, offset })
    }
}

#[derive(Debug)]
pub enum ThumbInstr {
    /// Memory load/store
    Mls(ThumbMls),
    Alu(ThumbAlu),
    /// (Conditional) Branch
    Branch(ThumbBranch),
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
        } else if (value >> 13) & 0b111 == 0b000 {
            // TODO: THUMB.2: add/subtract needs to be before this
            Ok(ThumbInstr::Alu(ThumbAlu::try_from(value)?))
        } else if (value >> 12) & 0b1111 == 0b1101 {
            Ok(ThumbInstr::Branch(ThumbBranch::try_from(value)?))
        } else {
            Err(ExecErr::UnknownThumbInstr(value))
        }
    }
}
