use crate::cpu::instruction::opcode::{Opcode, Mnemonic, NumArgs, OpcodeFlags};
use enumflags2::BitFlags;
use crate::cpu::{Regs, CPU};
use crate::cpu::instruction::args::{DstArg, Size};
use std::fmt::Formatter;

pub mod opcode;
pub mod actions;
pub mod data;
pub mod args;

#[derive(Clone)]
pub struct Instruction {
    pub flags: BitFlags<OpcodeFlags>,
    pub segment: Regs,
    pub action: Option<opcode::OpcodeAction>,
    pub mnemonic: Option<Mnemonic>,
    pub src: Option<args::DstArg>,
    pub dst: Option<args::DstArg>,
    pub reg_bits: u8,
    pub length: usize,
    pub next_cycles: usize
}

impl Instruction {
    pub fn exec(self, comp: &mut CPU) -> usize {
        let arg = self.clone();
        (self.action.unwrap())(comp, arg)
    }

    pub fn has_flag(&self, flag: OpcodeFlags) -> bool {
        self.flags.contains(flag)
    }

    pub fn new() -> Self {
        Self {
            flags: BitFlags::empty(),
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

    fn get_num_args(&self) -> NumArgs {
        let arg1 = if let Some(_) = self.dst { true } else { false };
        let arg2 = if let Some(_) = self.src { true } else { false };
        if arg1 && arg2 {
            NumArgs::Two
        } else if arg1 || arg2 {
            NumArgs::One
        } else {
            NumArgs::Zero
        }
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Opcode")
            .field("flags", &self.flags)
            .field("segment", &self.segment)
            .field("dst", &self.dst)
            .field("src", &self.src)
            .field("reg_bits", &self.reg_bits)
            .field("length", &self.length)
            .finish()
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.get_num_args() {
            NumArgs::Zero => write!(f, "{}", self.mnemonic.clone().unwrap().get(self.reg_bits)),
            NumArgs::One => write!(f, "{} {}", self.mnemonic.clone().unwrap().get(self.reg_bits), self.dst.clone().unwrap()),
            NumArgs::Two => write!(f,"{} {}, {}", self.mnemonic.clone().unwrap().get(self.reg_bits), self.dst.clone().unwrap(), self.src.clone().unwrap())
        }
    }
}

pub struct InstructionDecoder<'a> {
    opcodes: [Option<Opcode>; 256],
    ram: &'a [u8],
    ip: usize,
    next_cycles: usize,
    opcode_data: Option<Opcode>,
    s: u8,
    d: u8,
    instruction: Instruction
}

impl<'a> InstructionDecoder<'a> {
    pub fn new(opcodes: [Option<Opcode>; 256], ram: &'a[u8]) -> Self {
        Self {
            opcodes,
            ram,
            ip: 0,
            next_cycles: 0,
            opcode_data: None,
            s: 0,
            d: 0,

            instruction: Instruction::new()
        }
    }

    pub fn get(mut self) -> Instruction {
        let code = self.read_ip();
        let opcode_data = match self.get_opcode(code) {
            Some(op) => op,
            None => return self.instruction
        };

        self.instruction.flags = opcode_data.flags;
        self.instruction.action = Some(opcode_data.action.clone());
        self.instruction.segment = opcode_data.segment;
        self.instruction.mnemonic = Some(opcode_data.mnemonic.clone());

        self.opcode_data.replace(opcode_data);

        self.d = (code & 0x02) >> 1;
        self.s = if self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceDWord) { 2 } else { code & 0x01 };

        self.translate_placeholder();

        let has_src = if let Some(_) = self.instruction.src { true } else { false };
        let has_dst = if let Some(_) = self.instruction.dst { true } else { false };

        if !has_dst || !has_src {
            self.get_args();
        }

        self.instruction.length = self.ip;

        self.instruction
    }

    fn get_opcode(&self, code: u8) -> Option<Opcode> {
        Self::get_opcode_from_slice(&self.opcodes, code)
    }

    pub fn get_opcode_from_slice(opcodes: &[Option<Opcode>], opcode: u8) -> Option<Opcode> {
        match opcodes[opcode as usize].clone() {
            Some(op) => Some(op),
            None => match opcodes[(opcode & 0xFE) as usize].clone() {
                Some(op) => Some(op),
                None => match opcodes[(opcode & 0xFD) as usize].clone() {
                    Some(op) => Some(op),
                    None => Some(opcodes[(opcode & 0xFC) as usize].clone()?)
                }
            }
        }
    }

    fn translate_placeholder(&mut self) {
        if self.opcode_data.clone().unwrap().has_shorthand() {
            if let (Some(opcode::Placeholder::Imm), Some(opcode::Placeholder::Imm)) = (self.opcode_data.clone().unwrap().shorthand1, self.opcode_data.clone().unwrap().shorthand2) {
                if self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::SizeMismatch) {
                    self.instruction.dst = Some(args::DstArg::Imm16(self.read_ip_word()));
                    self.instruction.src = Some(args::DstArg::Imm8(self.read_ip()));
                } else {
                    if self.s == 0 {
                        self.instruction.dst = Some(args::DstArg::Imm8(self.read_ip()));
                        self.instruction.src = Some(args::DstArg::Imm8(self.read_ip()));
                    }
                }
            } else {
                let mut arg1_translated = None;
                let mut arg2_translated = None;
                if let Some(arg1) = self.opcode_data.clone().unwrap().shorthand1 {
                    arg1_translated.replace(self.translate_shorthand(arg1));
                }

                self.s = match arg1_translated.clone() {
                    Some(DstArg::Reg8(_)) => 0,
                    Some(DstArg::Reg16(_)) => 1,
                    _ => self.s
                };

                if let Some(arg2) = self.opcode_data.clone().unwrap().shorthand2 {
                    arg2_translated.replace(self.translate_shorthand(arg2));
                }

                let one_arg = if let opcode::NumArgs::Two = self.opcode_data.clone().unwrap().num_args { false } else { true };

                if (self.d == 1 && !self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::Immediate) && !one_arg) ||
                    self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceDirection) {
                    self.instruction.src = arg1_translated;
                    self.instruction.dst = arg2_translated;
                } else {
                    self.instruction.src = arg2_translated;
                    self.instruction.dst = arg1_translated;
                }
            }
        }
    }

    fn get_args(&mut self) {
        match self.opcode_data.clone().unwrap().num_args {
            NumArgs::Two => if !self.opcode_data.as_ref().unwrap().has_shorthand() { self.get_two_args() },
            NumArgs::One => if let None = self.instruction.dst { self.get_one_arg() },
            NumArgs::Zero => ()
        }
    }

    fn get_two_args(&mut self) {
        let immediate = self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::Immediate);
        let force_dword = self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceDWord);
        let force_word = self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceWord);
        let force_byte =self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceByte);

        let mod_reg_rm = self.read_ip();
        let (mod_bits, reg_bits, rm_bits) = Self::get_mod_reg_rm_bits(mod_reg_rm);
        self.instruction.reg_bits = reg_bits;

        let arg2 = if let None = self.instruction.src {
            Some(self.translate_mod_rm(mod_bits, rm_bits))
        } else {
            None
        };

        let arg1 = if immediate {
            if force_dword {
                DstArg::Imm32(self.read_ip_dword())
            } else if ((self.s == 1 && !self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::SizeMismatch))
                || force_word) &&
                !force_byte {
                DstArg::Imm16(self.read_ip_word())
            } else {
                DstArg::Imm8(self.read_ip())
            }
        } else if force_dword {
            DstArg::Ptr(self.read_ip_word(), Size::DWord)
        } else {
            DstArg::reg_to_arg(reg_bits, self.s)
        };

        if (self.d == 0 || immediate || force_dword) && !self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceDirection) {
            if let None = self.instruction.src {
                self.instruction.src.replace(arg1);
            }
            if let None = self.instruction.dst {
                self.instruction.dst = arg2;
            }
        } else {
            if let None = self.instruction.src {
                self.instruction.src = arg2;
            }
            if let None = self.instruction.dst {
                self.instruction.dst.replace(arg1);
            }
        }
    }

    fn get_one_arg(&mut self) {
        let immediate = self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::Immediate);
        let force_dword = self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceDWord);
        let force_word = self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceWord);
        let force_byte = self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::ForceByte);

        if let None = self.instruction.clone().dst {
            if immediate {
                let new_dst = if force_dword {
                    DstArg::Imm32(self.read_ip_dword())
                } else if ((self.d == 0 && !self.opcode_data.clone().unwrap().flags.contains(opcode::OpcodeFlags::SizeMismatch)) || force_word) && ! force_byte {
                    DstArg::Imm16(self.read_ip_word())
                } else {
                    DstArg::Imm8(self.read_ip())
                };
                self.instruction.dst.replace(new_dst);
            } else {
                let mod_reg_rm = self.read_ip();
                let (mod_bits, reg_bits, rm_bits) = Self::get_mod_reg_rm_bits(mod_reg_rm);
                self.instruction.reg_bits = reg_bits;

                let new_dst = self.translate_mod_rm(mod_bits, rm_bits);
                self.instruction.dst.replace(new_dst);
            }
        }
    }

    fn get_mod_reg_rm_bits(mod_reg_rm: u8) -> (u8, u8, u8) {
        ((mod_reg_rm & 0xC0) >> 6, (mod_reg_rm & 0x38) >> 3, (mod_reg_rm & 0x07) >> 0)
    }

    fn translate_mod_rm(&mut self, mod_bits: u8, rm_bits: u8) -> DstArg {
        if mod_bits == 0b00 && rm_bits == 0b110 {
            if self.s == 1 { DstArg::Ptr(self.read_ip_word(), Size::Word) } else { DstArg::Ptr(self.read_ip_word(), Size::Byte) }
        } else {
            let (reg1, reg2) = match rm_bits {
                0b000 => (Regs::BX, Some(Regs::SI)),
                0b001 => (Regs::BX, Some(Regs::DI)),
                0b010 => (Regs::BP, Some(Regs::SI)),
                0b011 => (Regs::BP, Some(Regs::DI)),
                0b100 => (Regs::SI, None),
                0b101 => (Regs::DI, None),
                0b110 => (Regs::BP, None),
                0b111 => (Regs::BX, None),
                _ => panic!("Impossible rm_bits value")
            };

            let offset = match mod_bits {
                0b00 => None,
                0b01 => Some(self.read_ip() as u16),
                0b10 => Some(self.read_ip_word()),
                0b11 => return DstArg::reg_to_arg(rm_bits, self.s),
                _ => panic!("Invalid mod_bits value")
            };

            match reg2 {
                Some(reg) => match offset { Some(off) => DstArg::RegPtrOffImm(reg1, reg, off, Size::from_s(self.s)), None => DstArg::RegPtrOff(reg1, reg, Size::from_s(self.s)) }
                None => match offset { Some(off) => DstArg::RegPtrImm(reg1, off, Size::from_s(self.s)), None => DstArg::RegPtr(reg1, Size::from_s(self.s)) }
            }
        }
    }

    fn translate_shorthand(&mut self, placeholder: opcode::Placeholder) -> DstArg {
        match placeholder {
            opcode::Placeholder::Reg(reg) => {
                if self.s == 1 {
                    DstArg::Reg16(reg)
                } else {
                    DstArg::Reg8(reg)
                }
            }
            opcode::Placeholder::RegEnum(reg) => {
                DstArg::Reg(reg)
            }
            opcode::Placeholder::Imm => {
                if self.s == 1 {
                    DstArg::Imm16(self.read_ip_word())
                } else {
                    DstArg::Imm8(self.read_ip())
                }
            }
            opcode::Placeholder::Reg8(reg) => DstArg::Reg8(reg),
            opcode::Placeholder::Reg16(reg) => DstArg::Reg16(reg),
            opcode::Placeholder::Byte(val) => DstArg::Imm8(val),
            opcode::Placeholder::Word(val) => DstArg::Imm16(val),
            opcode::Placeholder::Ptr => DstArg::Ptr(self.read_ip_word(), Size::Word)
        }
    }

    fn read_ip(&mut self) -> u8 {
        let tmp = self.ip;
        self.ip += 1;
        self.next_cycles += 1;
        self.ram[tmp]
    }

    fn read_ip_word(&mut self) -> u16 {
        (self.read_ip() as u16) | ((self.read_ip() as u16) << 8)
    }

    fn read_ip_dword(&mut self) -> u32 {
        (self.read_ip_word() as u32) | ((self.read_ip_word() as u32) << 16)
    }
}
