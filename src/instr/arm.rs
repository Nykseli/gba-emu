use super::common::{ExecErr, Register};

#[derive(Debug)]
pub enum Condition {
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
pub struct Branch {
    pub condition: Condition,
    /// is B or BL, true for Bl
    pub is_link: bool,
    pub nn: u32,
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
/// BX only, since BXJ AND BLX are not supported
pub struct BranchExchange {
    pub condition: Condition,
    pub rn: Register,
}

impl From<u32> for BranchExchange {
    fn from(value: u32) -> Self {
        let condition = Condition::from((value >> 28) & 0b1111);
        let rn = Register::from(value & 0b1111);
        Self { condition, rn }
    }
}

#[derive(Debug)]
pub enum AluOp {
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
pub struct Alu {
    pub condition: Condition,
    pub immediate: bool,
    pub op: AluOp,
    /// Data that's executed based in immediate mode
    pub operand: u32,
    pub s: bool,
    pub rn: Register,
    pub rd: Register,
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
pub struct Sdt {
    pub condition: Condition,
    pub immediate: bool,
    pub pre: bool,
    pub up: bool,
    pub bit: bool,
    pub tw: bool,
    pub load_memory: bool,
    pub rn: Register,
    pub rd: Register,
    /// Data that's executed based in immediate mode
    pub operand: u32,
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
pub enum Instruction {
    Branch(Branch),
    BranchExchange(BranchExchange),
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
        } else if (value >> 8) & 0xfffff == 0b0001_0010_1111_1111_1111 {
            Ok(Self::BranchExchange(BranchExchange::from(value)))
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
