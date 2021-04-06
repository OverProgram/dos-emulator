use crate::cpu::Regs;

#[derive(Clone, Copy, Debug)]
pub enum DstArg {
    Reg8(u8),
    Reg16(u8),
    Imm8(u8),
    Imm16(u16),
    Imm32(u32),
    Ptr32(u16),
    Ptr16(u16),
    Ptr8(u16),
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
            DstArg::Ptr32(val) => format!("[DWORD PTR {}]", val),
            DstArg::Ptr16(val) => format!("[WORD PTR {}]", val),
            DstArg::Ptr8(val) => format!("[BYTE PTR {}]", val),
            DstArg::Reg(reg) => reg.to_text(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SrcArg {
    Byte(u8),
    Word(u16),
    DWord(u32),
}
