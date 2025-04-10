pub enum ExecErr {
    UnknownInstr(u32),
    UnknownThumbInstr(u16),
    UnimplementedInstr(String),
}

pub type EResult<T> = Result<T, ExecErr>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
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

impl From<u16> for Register {
    fn from(value: u16) -> Self {
        (value as u32).into()
    }
}
