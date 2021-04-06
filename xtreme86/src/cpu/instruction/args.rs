use crate::cpu::{Regs, CPU};
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
    pub fn reg_to_arg(reg: u8, s: u8) -> Self {
        if s == 1 {
            Self::Reg16(reg)
        } else {
            Self::Reg8(reg)
        }
    }

    pub fn to_src_arg(self, comp: &mut CPU) -> Option<SrcArg> {
        match self {
            DstArg::Reg8(reg) => Some(SrcArg::Byte(comp.get_reg_8(reg)?)),
            DstArg::Reg16(reg) => Some(SrcArg::Word(comp.get_reg_16(reg)?)),
            DstArg::Imm8(val) => Some(SrcArg::Byte(val)),
            DstArg::Imm16(val) => Some(SrcArg::Word(val)),
            DstArg::Imm32(val) => Some(SrcArg::DWord(val)),
            DstArg::Ptr32(ptr) => Some(SrcArg::DWord(comp.read_mem_dword(ptr)?)),
            DstArg::Ptr16(ptr) => Some(SrcArg::Word(comp.read_mem_word_mut(ptr)?)),
            DstArg::Ptr8(ptr) => Some(SrcArg::Byte(comp.read_mem_byte_mut(ptr)?)),
            DstArg::Reg(reg) => Some(SrcArg::Word(comp.regs.get(&reg)?.value))
        }
    }
}

impl std::fmt::Display for DstArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
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
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SrcArg {
    Byte(u8),
    Word(u16),
    DWord(u32),
}
