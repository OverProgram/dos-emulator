use crate::cpu::instruction::opcode::{Opcode, Mnemonic};
use enumflags2::BitFlags;
use crate::cpu::{Regs, CPU};
use crate::cpu::instruction::args::DstArg;

mod opcode;
pub mod actions;
mod data;
mod args;

pub struct Instruction {
    // pub num_args: opcode::NumArgs,
    // pub flags: BitFlags<opcode::OpcodeFlags>,
    pub segment: Regs,
    pub action: Option<OpcodeAction>,
    pub mnemonic: Option<Mnemonic>,
    pub src: Option<args::DstArg>,
    pub dst: Option<args::DstArg>,
    pub reg_bits: u8,
    pub length: usize,
    pub next_cycles: usize
}

impl Instruction {
    fn new() -> Self {
        Self {
            segment: Regs::DS,
            action: None,
            mnemonic: None,
            src: None,
            dst: None,
            reg_bits: 0,
            length: 0,
            next_cycles: 0
        }
    }

    fn construct(opcode_data: &Opcode, d: u8, s: &mut u8, ram: &[u8], ip: &mut usize, next_cycles: &mut usize) -> Self {
        let mut instruction = Self::new();
        instruction.translate_placeholder(opcode_data, d, s, ram, ip, next_cycles);
        instruction.get_args();
        instruction
    }

    fn translate_placeholder(&mut self, opcode_data: &Opcode, d: u8, s: &mut u8, ram: &[u8], ip: &mut usize, next_cycles: &mut usize) {
        if opcode_data.has_shorthand() {
            if let (Some(opcode::Placeholder::Imm), Some(opcode::Placeholder::Imm)) = (opcode_data.shorthand1, opcode_data.shorthand2) {
                if opcode_data.flags.contains(opcode::OpcodeFlags::SizeMismatch) {
                    self.dst = Some(args::DstArg::Imm16(InstructionDecoder::read_ip_word(ram, ip, next_cycles)));
                    self.src = Some(args::DstArg::Imm8(InstructionDecoder::read_ip(ram, ip, next_cycles)));
                } else {
                    if s == 0 {
                        self.dst = Some(args::DstArg::Imm8(InstructionDecoder::read_ip(ram, ip, next_cycles)));
                        self.src = Some(args::DstArg::Imm8(InstructionDecoder::read_ip(ram, ip, next_cycles)));
                    }
                }
            } else {
                let mut arg1_translated = None;
                let mut arg2_translated = None;
                if let Some(arg1) = opcode_data.shorthand1 {
                    arg1_translated.replace(Self::translate_shorthand(arg1, *s, ram, ip, next_cycles));
                }

                if let Some(arg2) = opcode_data.shorthand2 {
                    arg2_translated.replace(Self::translate_shorthand(arg2, *s, ram, ip, next_cycles));
                }

                *s = match arg1_translated.clone() {
                    Some(DstArg::Reg8(_)) => 0,
                    Some(DstArg::Reg16(_)) => 1,
                    _ => s
                };

                let one_arg = if let opcode::NumArgs::Two = opcode_data.num_args { false } else { true };

                if (d == 1 && !opcode_data.flags.contains(opcode::OpcodeFlags::Immediate) && !one_arg) ||
                    opcode_data.flags.contains(opcode::OpcodeFlags::ForceDirection) {
                    self.src = arg1_translated;
                    self.dst = arg2_translated;
                } else {
                    self.src = arg2_translated;
                    self.dst = arg1_translated;
                }
            }
        }
    }

    fn get_args(&mut self) {

    }

    fn translate_shorthand(placeholder: opcode::Placeholder, s: u8, ram: &[u8], ip: &mut usize, next_cycles: &mut usize) -> DstArg {
        match placeholder {
            opcode::Placeholder::Reg(reg) => {
                if s == 1 {
                    DstArg::Reg8(reg)
                } else {
                    DstArg::Reg16(reg)
                }
            }
            opcode::Placeholder::Imm => {
                if s == 1 {
                    DstArg::Imm16(InstructionDecoder::read_ip_word(ram, ip, next_cycles))
                } else {
                    DstArg::Imm8(InstructionDecoder::read_ip(ram, ip, next_cycles))
                }
            }
            opcode::Placeholder::Reg8(reg) => DstArg::Reg8(reg),
            opcode::Placeholder::Reg16(reg) => DstArg::Reg8(reg),
            opcode::Placeholder::Byte(val) => DstArg::Imm8(val),
            opcode::Placeholder::Word(val) => DstArg::Imm16(val),
            opcode::Placeholder::Ptr => DstArg::Ptr16(InstructionDecoder::read_ip_word(ram, ip, next_cycles))
        }
    }
}

pub struct InstructionDecoder {
    opcode_data: [Option<Opcode>; 256],
}

impl InstructionDecoder {
    pub fn new() -> Self {
        Self {
            opcode_data: Opcode::get_opcode_data()
        }
    }

    pub fn decode(&self, ram: &[u8]) -> Instruction {
        let mut ip = 0;
        let mut next_cycles = 0;
        let code = Self::read_ip(ram, &mut ip, &mut next_cycles);
        let opcode_data = match self.opcode_data[code] {
            Some(op) => op,
            None => return Instruction::new()
        };

        let d = (code & 0x02) >> 1;
        let mut s = if force_dword { 2 } else { code & 0x01 };

        let mut instruction = Instruction::new();

        instruction.translate_placeholder(&opcode_data, d, &mut s, ram, &mut ip, &mut next_cycles);

        instruction
    }

    // fn get_opcode(&self, code: u8) -> Result<Opcode, ()> {
    //     self.opcode_data[code].map_or(Err(()), |op| Ok(op))
    // }

    fn read_ip(ram: &[u8], ip: &mut usize, next_cycles: &mut usize) -> u8 {
        let tmp = *ip;
        *ip += 1;
        *next_cycles += 1;
        ram[tmp]
    }

    fn read_ip_word(ram: &[u8], ip: &mut usize, next_cycles: &mut usize) -> u16 {
        (Self::read_ip(ram, ip, next_cycles) as u16) | ((Self::read_ip(ram, ip, next_cycles) as u16) << 8)
    }
}
