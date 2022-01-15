use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Register {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    //HL = 6,
    A = 7
}

#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum RegisterPair {
    BC = 0,
    DE = 1,
    HL = 2,
    AF = 3
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Opcode {
    Unknown,
    LdRR(Register, Register),
    LdRImm(Register),
    LdRMem(Register, RegisterPair),
    LdMemR(RegisterPair, Register),
    LdMemImm(RegisterPair),
}

impl Opcode {
    pub fn from_u8(op: u8) -> Self {
        match &op {
            0x40..=0x45 | 0x47..=0x4D | 0x4F..=0x55 |
            0x57..=0x5D | 0x5F..=0x65 | 0x67..=0x6D |
            0x6F..=0x6F | 0x78..=0x7D | 0x7F => {
                let target = Register::try_from((op & 0x070) >> 3).unwrap();
                let source = Register::try_from(op & 0o007).unwrap();
                Self::LdRR(target, source)
            },
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                let target = Register::try_from((op & 0o070) >> 3).unwrap();
                Self::LdRImm(target)
            },
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x76 | 0x7E => {
                let target = Register::try_from((op & 0o070) >> 3).unwrap();
                Self::LdRMem(target, RegisterPair::HL)
            },
            0x70..=0x77 => {
                let source = Register::try_from(op & 0o007).unwrap();
                Self::LdMemR(RegisterPair::HL, source)
            },
            0x36 => {
                Self::LdMemImm(RegisterPair::HL)
            }
            _ => Self::Unknown
        }
    }

    pub fn cycles(&self) -> u8 {
        match self {
            Self::Unknown => 1,
            Self::LdRR(_, _) => 1,
            Self::LdRImm(_) => 2,
            Self::LdRMem(_, _) => 2,
            Self::LdMemR(_, _) => 2,
            Self::LdMemImm(_) => 3,
        }
    }
}
