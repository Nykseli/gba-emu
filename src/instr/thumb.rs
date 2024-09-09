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
pub enum ThumbInstr {
    /// Memory load/store
    Mls(ThumbMls),
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
        } else {
            Err(ExecErr::UnknownThumbInstr(value))
        }
    }
}
