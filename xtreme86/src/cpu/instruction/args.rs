use crate::cpu::Regs;
use std::fmt::Formatter;

#[derive(Copy, Clone, Debug)]
pub enum Size {
    Byte,
    Word,
    DWord
}

impl Size {
    pub fn from_s(s: u8) -> Self {
        match s {
            0 => Self::Byte,
            1 => Self::Word,
            2 => Self::DWord,
            _ => panic!("Invalid s value")
        }
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Byte => "byte",
            Self::Word => "word",
            Self::DWord => "dword"
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DstArg {
    Reg8(u8),
    Reg16(u8),
    Imm8(u8),
    Imm16(u16),
    Imm32(u32),
    Ptr(u16, Size),
    RegPtr(Regs, Size),
    RegPtrImm(Regs, u16, Size),
    RegPtrOff(Regs, Regs, Size),
    RegPtrOffImm(Regs, Regs, u16, Size),
    Reg(Regs)
}

impl DstArg {
    pub fn to_text(&self) -> String {
        match self {
            DstArg::Reg8(id) => Regs::id_8_bit_to_text(*id),
            DstArg::Reg16(id) => Regs::translate_reg16(*id).unwrap().to_text(),
            DstArg::Imm8(val) => val.to_string(),
            DstArg::Imm16(val) => val.to_string(),
            DstArg::Imm32(val) => val.to_string(),
            DstArg::Ptr(val, size) => format!("{} [{}]", size, val),
            DstArg::RegPtr(reg, size) => format!("{} [{}]", size, reg.to_text()),
            DstArg::RegPtrImm(reg, imm, size) => format!("{} [{} + {}]", size, reg.to_text(), imm),
            DstArg::RegPtrOff(reg, off_reg, size) => format!("{} [{} + {}]", size, reg.to_text(), off_reg.to_text()),
            DstArg::RegPtrOffImm(reg, off_reg, imm, size) => format!("{} [{} + {} + {}]", size, reg.to_text(), off_reg.to_text(), imm),
            DstArg::Reg(reg) => reg.to_text(),
        }
    }

    pub fn reg_to_arg(reg: u8, s: u8) -> Self {
        if s == 1 {
            Self::Reg16(reg)
        } else {
            Self::Reg8(reg)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SrcArg {
    Byte(u8),
    Word(u16),
    DWord(u32),
}
