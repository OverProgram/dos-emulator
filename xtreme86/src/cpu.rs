mod reg;
mod mem;
mod alu;
mod stack;
mod jmp;
mod int;
mod flags;

use std::collections::HashMap;
use std::rc::Rc;
use enumflags2::BitFlags;
use std::fmt::{Debug, Formatter};
use std::fmt;
use crate::cpu::reg::Reg;

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
enum OpcodeFlags {
    Immediate = 0x0001,
    SizeMismatch = 0x0002,
    Nop = 0x0004,
    ForceWord = 0x0008,
    ForceByte = 0x0010,
    ForceDWord = 0x0020,
    ForceDirection = 0x0040,
}

pub struct CPUFlags ;

impl CPUFlags {
    pub const CARRY: u16 = 0x0001;
    pub const PARITY: u16 = 0x0040;
    pub const AUX_CARRY: u16 = 0x0010;
    pub const ZERO: u16 = 0x0040;
    pub const SIGN: u16 = 0x0080;
    pub const TRAP: u16 = 0x0100;
    pub const INTERRUPT: u16 = 0x0200;
    pub const DIRECTION: u16 = 0x0400;
    pub const OVERFLOW: u16 = 0x0800;
}

mod exceptions {
    pub const DIVIDE_BY_ZERO: u8 = 0x00;
    pub const SINGLE_STEP_INSTRUCTION: u8 = 0x01;
    pub const NMI: u8 = 0x02;
    pub const BREAKPOINT: u8 = 0x03;
    pub const INTO: u8 = 0x04;
    pub const BOUND: u8 = 0x05;
    pub const INVALID_OPCODE: u8 = 0x06;
    pub const NO_EXTENSION: u8 = 0x07;
    pub const IVT_TOO_SMALL: u8 = 0x08;
}

#[derive(Clone, Copy, Debug)]
enum NumArgs {
    Zero,
    One,
    Two
}

#[derive(Clone, Copy, Debug)]
enum DstArg {
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
    fn to_text(&self) -> String {
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
enum SrcArg {
    Byte(u8),
    Word(u16),
    DWord(u32),
}

#[derive(Clone, Copy, Debug)]
enum Placeholder {
    Reg8(u8),
    Reg16(u8),
    Reg(u8),
    Byte(u8),
    Word(u16),
    Imm,
    Ptr
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum Regs {
    AX,
    BX,
    CX,
    DX,
    SI,
    DI,
    SP,
    BP,
    ES,
    CS,
    SS,
    DS,
    IP,
    FLAGS
}

impl Regs {
    fn to_text(&self) -> String {
        String::from(match self {
            Regs::AX => "AX",
            Regs::BX => "BX",
            Regs::CX => "CX",
            Regs::DX => "DX",
            Regs::SI => "SI",
            Regs::DI => "DI",
            Regs::SP => "SP",
            Regs::BP => "BP",
            Regs::ES => "ES",
            Regs::CS => "CS",
            Regs::SS => "SS",
            Regs::DS => "DS",
            Regs::IP => "IP",
            Regs::FLAGS => "FLAGS"
        })
    }

    fn id_8_bit_to_text(id: u8) -> String {
        String::from(match id {
            0 => "AL",
            1 => "CL",
            2 => "DL",
            3 => "BL",
            4 => "AH",
            5 => "CH",
            6 => "DH",
            7 => "BH",
            _ => ""
        })
    }

    fn translate_reg16(num: u8) -> Option<Self> {
        match num {
            0 => Some(Regs::AX),
            1 => Some(Regs::CX),
            2 => Some(Regs::DX),
            3 => Some(Regs::BX),
            4 => Some(Regs::SP),
            5 => Some(Regs::BP),
            6 => Some(Regs::SI),
            7 => Some(Regs::DI),
            _ => None
        }
    }

    fn translate_reg8(num: u8) -> Option<(Self, WordPart)> {
        Some((Self::translate_reg16(num % 4)?, if num % 2 == 0 { WordPart::Low } else { WordPart::High }))
    }
}

#[derive(Copy, Clone, Debug)]
enum WordPart {
    Low,
    High
}

#[derive(Clone)]
struct Opcode {
    instruction: Rc<dyn Fn(&mut CPU) -> usize>,
    num_args: NumArgs,
    cycles: usize,
    shorthand: Option<(Placeholder, Option<Placeholder>)>,
    flags: BitFlags<OpcodeFlags>,
    segment: Regs,
    mnemonic: Rc<dyn Fn(u8) -> Option<String>>
}

impl Opcode {
    fn new(instruction: Rc<dyn Fn(&mut CPU) -> usize>, mnemonic: Rc<dyn Fn(u8) -> Option<String>>, num_args: NumArgs, cycles: usize, shorthand: Option<(Placeholder, Option<Placeholder>)>, segment: Regs, flags: BitFlags<OpcodeFlags>) -> Self {
        Self {
            instruction,
            num_args,
            cycles,
            shorthand,
            flags,
            segment,
            mnemonic
        }
    }

    fn has_flag(&self, flag: BitFlags<OpcodeFlags>) -> bool {
        self.flags.contains(flag)
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Opcode")
            .field("num_args", &self.num_args)
            .field("cycles", &self.cycles)
            .field("shorthand", &self.shorthand)
            .field("flags", &self.flags)
            .field("segment", &self.segment)
            .finish()
    }
}

#[derive(Debug)]
pub struct CPU {
    ram: Vec<u8>,
    regs: HashMap<Regs, reg::Reg>,
    opcodes: HashMap<u8, Opcode>,
    instruction: Option<Opcode>,
    src: Option<SrcArg>,
    dst: Option<DstArg>,
    seg: Regs,
    next_cycles: usize,
    reg_bits: u8,
    irq: Option<u8>,
    opcode_address: (u16, u16),
    src_ptr: Option<u16>,
}

impl CPU {
    pub fn new(ram_size: usize) -> Self {
        // Create and allocate the ram
        let ram: Vec<u8> = vec![0; ram_size];

        // Create register HashMap
        let mut regs: HashMap<Regs, reg::Reg> = HashMap::new();
        regs.insert(Regs::AX, reg::Reg::new());
        regs.insert(Regs::BX, reg::Reg::new());
        regs.insert(Regs::CX, reg::Reg::new());
        regs.insert(Regs::DX, reg::Reg::new());
        regs.insert(Regs::SI, reg::Reg::new());
        regs.insert(Regs::DI, reg::Reg::new());
        regs.insert(Regs::BP, reg::Reg::new());
        regs.insert(Regs::SP, reg::Reg::new());
        regs.insert(Regs::ES, reg::Reg::new());
        regs.insert(Regs::DS, reg::Reg::new());
        regs.insert(Regs::SS, reg::Reg::new());
        regs.insert(Regs::CS, reg::Reg::new());
        regs.insert(Regs::IP, reg::Reg::new());
        regs.insert(Regs::FLAGS, reg::Reg::new());

        // Define opcodes
        let mut opcodes: HashMap<u8, Opcode> = HashMap::new();
        //NOP
        opcodes.insert(0x90, Opcode::new(Rc::new(mem::nop), Rc::new(mem::nop_mnemonic), NumArgs::Zero, 1, None, Regs::DS, OpcodeFlags::Nop.into()));
        // Move opcodes
        opcodes.insert(0x88, Opcode::new(Rc::new(mem::mov), Rc::new(mem::mov_mnemonic), NumArgs::Two, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0xA0, Opcode::new(Rc::new(mem::mov), Rc::new(mem::mov_mnemonic), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Ptr))), Regs::DS, BitFlags::empty()));
        for x in 0..7 {
            opcodes.insert(0xB0 + x, Opcode::new(Rc::new(mem::mov), Rc::new(mem::mov_mnemonic), NumArgs::Two, 1, Some((Placeholder::Reg8(x), Some(Placeholder::Imm))),Regs::DS, OpcodeFlags::Immediate.into()));
            opcodes.insert(0xB8 + x, Opcode::new(Rc::new(mem::mov), Rc::new(mem::mov_mnemonic), NumArgs::Two, 1, Some((Placeholder::Reg16(x), Some(Placeholder::Imm))), Regs::DS, OpcodeFlags::Immediate.into()));
        }
        opcodes.insert(0xC6, Opcode::new(Rc::new(mem::mov), Rc::new(mem::mov_mnemonic), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::Immediate.into()));
        opcodes.insert(0xC5, Opcode::new(Rc::new(mem::ldw), Rc::new(mem::lds_mnemonic), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::ForceDWord.into()));
        opcodes.insert(0xC4, Opcode::new(Rc::new(mem::ldw), Rc::new(mem::les_mnemonic), NumArgs::Two, 1, None, Regs::ES, OpcodeFlags::ForceDWord.into()));
        opcodes.insert(0x8D, Opcode::new(Rc::new(mem::lea), Rc::new(mem::lea_mnemonic), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::ForceDirection.into()));
        opcodes.insert(0xAC, Opcode::new(Rc::new(mem::lods), Rc::new(mem::lods_mnemonic), NumArgs::One, 1, Some((Placeholder::Byte(0), None)), Regs::DS, BitFlags::empty()));
        opcodes.insert(0xAD, Opcode::new(Rc::new(mem::lods), Rc::new(mem::lods_mnemonic), NumArgs::One, 1, Some((Placeholder::Word(0), None)), Regs::DS, BitFlags::empty()));
        // Conversion
        opcodes.insert(0x98, Opcode::new(Rc::new(mem::cbw), Rc::new(mem::cbw_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0x99, Opcode::new(Rc::new(mem::cdw), Rc::new(mem::cdw_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        // ALU opcodes
        let mut alu_opcodes: Vec<(Rc<dyn Fn(&mut CPU) -> usize>, Rc<dyn Fn(u8) -> Option<String>>, u8)> = Vec::new();
        alu_opcodes.push((Rc::new(alu::add), Rc::new(alu::add_mnemonic), 0x00));
        alu_opcodes.push((Rc::new(alu::sub), Rc::new(alu::sub_mnemonic), 0x28));
        alu_opcodes.push((Rc::new(alu::xor), Rc::new(alu::xor_mnemonic), 0x30));
        alu_opcodes.push((Rc::new(alu::and), Rc::new(alu::and_mnemonic), 0x20));
        alu_opcodes.push((Rc::new(alu::or), Rc::new(alu::or_mnemonic), 0x08));
        for (instruction, mnemonic, offset) in alu_opcodes.into_iter() {
            opcodes.insert(0x00 + offset, Opcode::new(instruction.clone(), mnemonic.clone(), NumArgs::Two, 1, None, Regs::DS, BitFlags::empty()));
            opcodes.insert(0x04 + offset, Opcode::new(instruction.clone(), mnemonic.clone(), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Imm))), Regs::DS, OpcodeFlags::Immediate.into()));
        }
        opcodes.insert(0x80, Opcode::new(Rc::new(alu::alu_dispatch_two_args), Rc::new(alu::alu_dispatch_two_args_mnemonic), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::Immediate.into()));
        for x in 0..7 {
            opcodes.insert(0x40 + x, Opcode::new(Rc::new(alu::inc),Rc::new(alu::inc_mnemonic), NumArgs::Zero, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, BitFlags::empty()));
            opcodes.insert(0x48 + x, Opcode::new(Rc::new(alu::dec), Rc::new(alu::dec_mnemonic), NumArgs::Zero, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, BitFlags::empty()));
        }
        opcodes.insert(0x83, Opcode::new(Rc::new(alu::alu_dispatch_two_args), Rc::new(alu::alu_dispatch_two_args_mnemonic), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::Immediate | OpcodeFlags::SizeMismatch));
        opcodes.insert(0xFE, Opcode::new(Rc::new(alu::alu_dispatch_one_arg), Rc::new(alu::alu_dispatch_one_arg_mnemonic), NumArgs::One, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0xF6, Opcode::new(Rc::new(alu::mul_dispatch), Rc::new(alu::mul_dispatch_mnemonic), NumArgs::One, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0x37, Opcode::new(Rc::new(alu::aaa), Rc::new(alu::aaa_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0xD5, Opcode::new(Rc::new(alu::aad), Rc::new(alu::aad_mnemonic), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::Immediate | OpcodeFlags::ForceByte));
        opcodes.insert(0x3F, Opcode::new(Rc::new(alu::aas), Rc::new(alu::aas_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0x27, Opcode::new(Rc::new(alu::daa), Rc::new(alu::daa_mnemonic),NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0x14, Opcode::new(Rc::new(alu::adc), Rc::new(alu::adc_mnemonic), NumArgs::Two, 1, Some((Placeholder::Reg8(0), Some(Placeholder::Imm))), Regs::DS, OpcodeFlags::Immediate.into()));
        opcodes.insert(0x10, Opcode::new(Rc::new(alu::adc), Rc::new(alu::adc_mnemonic), NumArgs::Two, 1, None, Regs::DS, BitFlags::empty()));
        // Stack opcodes
        for x in 0..7 {
            opcodes.insert(0x50 + x, Opcode::new(Rc::new(stack::push), Rc::new(stack::push_mnemonic), NumArgs::One, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, BitFlags::empty()));
            opcodes.insert(0x58 + x, Opcode::new(Rc::new(stack::pop), Rc::new(stack::pop_mnemonic), NumArgs::One, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, BitFlags::empty()));
        }
        opcodes.insert(0x8F, Opcode::new(Rc::new(stack::pop), Rc::new(stack::pop_mnemonic), NumArgs::One, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0xE8, Opcode::new(Rc::new(stack::near_call), Rc::new(stack::call_mnemonic), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::Immediate | OpcodeFlags::ForceWord));
        opcodes.insert(0x9A, Opcode::new(Rc::new(stack::far_call), Rc::new(stack::call_mnemonic), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::Immediate | OpcodeFlags::ForceWord));
        opcodes.insert(0xC3, Opcode::new(Rc::new(stack::ret), Rc::new(stack::ret_mnemonic), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::Immediate | OpcodeFlags::ForceWord));
        opcodes.insert(0xC8, Opcode::new(Rc::new(stack::enter), Rc::new(stack::enter_mnemonic), NumArgs::Two, 1, Some((Placeholder::Imm, Some(Placeholder::Imm))), Regs::DS, OpcodeFlags::Immediate | OpcodeFlags::SizeMismatch));
        opcodes.insert(0xC9, Opcode::new(Rc::new(stack::leave), Rc::new(stack::leave_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        // Jump opcodes
        opcodes.insert(0xE9, Opcode::new(Rc::new(jmp::jmp), Rc::new(jmp::jmp_mnemonic), NumArgs::One, 1, None, Regs::CS, OpcodeFlags::Immediate.into()));
        let flag_condition: Vec<(Box<dyn Fn(&Self) -> bool>, String)> = vec![(Box::new(|this: &Self| this.check_flag(CPUFlags::OVERFLOW)), String::from("O")), (Box::new(|this: &Self| {!this.check_flag(CPUFlags::OVERFLOW)}), String::from("NO")), (Box::new(|this: &Self| {this.check_flag(CPUFlags::CARRY)}), String::from("C")),
                                                                                                                                                                (Box::new(|this: &Self| {!this.check_flag(CPUFlags::CARRY)}), String::from("C")), (Box::new(|this: &Self| {this.check_flag(CPUFlags::ZERO)}), String::from("E")), (Box::new(|this: &Self| {!this.check_flag(CPUFlags::ZERO)}), String::from("NE")),
                                                                                                                                                                (Box::new(|this: &Self| {this.check_flag(CPUFlags::CARRY) || this.check_flag(CPUFlags::ZERO)}), String::from("BE")), (Box::new(|this: &Self| {!this.check_flag(CPUFlags::CARRY) && !this.check_flag(CPUFlags::ZERO)}), String::from("A")), (Box::new(|this: &Self| {this.check_flag(CPUFlags::SIGN)}), String::from("S")),
                                                                                                                                                                (Box::new(|this: &Self| {!this.check_flag(CPUFlags::SIGN)}), String::from("NS")), (Box::new(|this: &Self| {this.check_flag(CPUFlags::PARITY)}), String::from("P")), (Box::new(|this: &Self| {this.check_flag(!CPUFlags::PARITY)}), String::from("NP")),
                                                                                                                                                                (Box::new(|this: &Self| {this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW)}), String::from("L")), (Box::new(|this: &Self| {!this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW)}), String::from("GE")), (Box::new(|this: &Self| {this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW) || this.check_flag(CPUFlags::ZERO)}), String::from("LE")),
                                                                                                                                                                (Box::new(|this: &Self| {this.check_flag(CPUFlags::SIGN) && !this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW)}), String::from("G"))];
        let mut i = 0;
        for (condition, cond_text) in flag_condition {
            opcodes.insert(0x70 + i, Opcode::new(jmp::cond_jmp(condition), jmp::cond_jmp_mnemonic(cond_text), NumArgs::One, 1, None, Regs::CS, OpcodeFlags::Immediate | OpcodeFlags::SizeMismatch));
            i += 1;
        }
        // Flag opcodes
        opcodes.insert(0xF8, Opcode::new(Rc::new(flags::clc), Rc::new(flags::clc_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0xFC, Opcode::new(Rc::new(flags::cld), Rc::new(flags::cld_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0xFA, Opcode::new(Rc::new(flags::cli), Rc::new(flags::cli_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0xF5, Opcode::new(Rc::new(flags::cmc), Rc::new(flags::cmc_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0x38, Opcode::new(Rc::new(flags::cmp), Rc::new(flags::cmp_mnemonic), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::SizeMismatch.into()));
        opcodes.insert(0x3C, Opcode::new(Rc::new(flags::cmp), Rc::new(flags::cmp_mnemonic), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Imm))), Regs::DS, OpcodeFlags::Immediate.into()));
        opcodes.insert(0xA6, Opcode::new(Rc::new(flags::cmps), Rc::new(flags::cmps_mnemonic), NumArgs::Zero, 1, Some((Placeholder::Reg16(6), Some(Placeholder::Reg16(7)))), Regs::DS, BitFlags::empty()));
        opcodes.insert(0x9F, Opcode::new(Rc::new(flags::lahf), Rc::new(flags::lahf_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        // Interrupt opcodes
        opcodes.insert(0xCD, Opcode::new(Rc::new(int::int_req), Rc::new(int::int_mnemonic), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::Immediate | OpcodeFlags::ForceByte));
        opcodes.insert(0xCC, Opcode::new(Rc::new(int::int_req), Rc::new(int::int_mnemonic), NumArgs::One, 1, Some((Placeholder::Byte(3), None)), Regs::DS, OpcodeFlags::Immediate.into()));
        opcodes.insert(0xCF, Opcode::new(Rc::new(int::iret), Rc::new(int::iret_mnemonic), NumArgs::Zero, 1, None, Regs::DS, BitFlags::empty()));
        opcodes.insert(0x62, Opcode::new(Rc::new(int::bound), Rc::new(int::bound_mnemonic), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::ForceDWord.into()));

        Self {
            ram,
            regs,
            opcodes,
            instruction: None,
            src: None,
            dst: None,
            seg: Regs::DS,
            next_cycles: 0,
            reg_bits: 0,
            irq: None,
            opcode_address: (0, 0),
            src_ptr: None
        }
    }

    pub fn step(&mut self) {
        if self.next_cycles > 0 {
            self.next_cycles -= 1;
        } else if let Some(opcode) = self.instruction.clone() {
            (opcode.instruction)(self);
            self.instruction = None;
            self.src = None;
            self.dst = None;
            self.src_ptr = None
        } else if let Some(_) = self.irq {
            self.next_cycles += int::int(self);
        } else {
            let opcode_address =  (self.regs[&Regs::CS].value, self.regs[&Regs::IP].value);
            self.opcode_address = opcode_address;
            let (instruction, dst, src, seg, next_cycles, ip_offset, reg_bits) = self.decode_instruction(self.regs.get(&Regs::IP).unwrap().value as usize);
            self.instruction = instruction;
            self.dst = dst;
            self.src_ptr = match src {
                Some(arg) => Self::get_ptr(arg),
                None => None
            };
            self.src = match src {
                Some(arg) => self.get_src_arg_mut(arg),
                None => None
            };
            self.reg_bits = reg_bits;
            self.next_cycles = next_cycles;
            self.seg = seg;
            self.regs.get_mut(&Regs::IP).unwrap().value = ip_offset as u16;
        }
    }

    fn decode_instruction(&self, loc: usize) -> (Option<Opcode>, Option<DstArg>, Option<DstArg>, Regs, usize, u16, u8) {
        let mut next_cycles = 0;
        let mut ip_tmp = loc;
        let code = self.read_ip(&mut ip_tmp, &mut next_cycles);
        let opcode = self.get_opcode(&code).clone();
        next_cycles += opcode.cycles;
        let immediate = opcode.has_flag(OpcodeFlags::Immediate.into());
        let size_mismatch = opcode.has_flag(OpcodeFlags::SizeMismatch.into());
        let force_dword = opcode.has_flag(OpcodeFlags::ForceDWord.into());
        let force_word = opcode.has_flag(OpcodeFlags::ForceWord.into());
        let force_byte = opcode.has_flag(OpcodeFlags::ForceByte.into());
        let force_direction = opcode.has_flag(OpcodeFlags::ForceDirection.into());
        let d = (code & 0x02) >> 1;
        let mut s = if force_dword { 2 } else { (code & 0x01) >> 0 };
        let num_args = opcode.num_args;
        let shorthand = opcode.shorthand.clone();
        let seg = opcode.segment;
        let mut src: Option<DstArg> = None;
        let mut dst: Option<DstArg> = None;
        let mut reg_bits = 0;

        if let Some((arg1, arg2)) = opcode.shorthand.clone() {
            if let (Placeholder::Imm, Some(Placeholder::Imm)) = (arg1, arg2) {
                if size_mismatch {
                    dst = Some(DstArg::Imm16(self.read_ip_word(&mut ip_tmp, &mut next_cycles)));
                    src = Some(DstArg::Imm8(self.read_ip(&mut ip_tmp, &mut next_cycles)));
                } else {
                    if s == 0 {
                        dst = Some(DstArg::Imm8(self.read_ip(&mut ip_tmp, &mut next_cycles)));
                        src = Some(DstArg::Imm8(self.read_ip(&mut ip_tmp, &mut next_cycles)));
                    } else if s ==2 {
                        dst = Some(DstArg::Imm16(self.read_ip_word(&mut ip_tmp, &mut next_cycles)));
                        src = Some(DstArg::Imm16(self.read_ip_word(&mut ip_tmp, &mut next_cycles)));
                    }
                }
            } else {
                let arg1_translated = Some(self.translate_placeholder(arg1, s, &mut ip_tmp, &mut next_cycles));
                s = match arg1_translated.clone().unwrap() {
                    DstArg::Reg8(_) => 0,
                    DstArg::Reg16(_) => 1,
                    _ => s
                };
                let mut arg2_translated = None;
                if let Some(arg) = arg2 {
                    arg2_translated = Some(self.translate_placeholder(arg, s, &mut ip_tmp, &mut next_cycles));
                }
                let one_arg = if let NumArgs::Two = num_args { false } else { true };
                if (d == 1 && !immediate && !one_arg) || force_direction {
                    src = arg1_translated;
                    dst = arg2_translated;
                } else {
                    src = arg2_translated;
                    dst = arg1_translated;
                }
            }
        }

        match num_args {
            NumArgs::Two => {
                if let None = shorthand {
                    let mod_reg_rm = self.read_ip(&mut ip_tmp, &mut next_cycles);
                    let rm = (mod_reg_rm & 0x07) >> 0;
                    let reg = (mod_reg_rm & 0x38) >> 3;
                    let mod_bits = (mod_reg_rm & 0xC0) >> 6;
                    let arg2 = {
                        if let None = self.src {
                            self.translate_mod_rm(mod_bits, rm, s, &mut ip_tmp, &mut next_cycles)
                        } else {
                            None
                        }
                    };
                    let arg1 = if immediate {
                        Some(
                            if force_dword {
                                DstArg::Imm32(self.read_ip_dword(&mut ip_tmp, &mut next_cycles))
                            } else if ((s == 1 && !size_mismatch) || force_word) && !force_byte {
                                DstArg::Imm16(self.read_ip_word(&mut ip_tmp, &mut next_cycles))
                            } else {
                                DstArg::Imm8(self.read_ip(&mut ip_tmp, &mut next_cycles))
                            }
                        )
                    } else if force_dword {
                        Some(DstArg::Ptr32(self.read_ip_word(&mut ip_tmp, &mut next_cycles)))
                    } else {
                        Some(Self::reg_to_arg(reg, s))
                    };

                    if (d == 0 || immediate || force_dword) && !force_direction {
                        if let None = self.src {
                            src = arg1;
                        }
                        if let None = self.dst {
                            dst = arg2;
                        }
                    } else {
                        if let None = self.src {
                            src = arg2;
                        }
                        if let None = self.dst {
                            dst = arg1;
                        }
                    }

                    reg_bits = reg;
                }
            },
            NumArgs::One => {
                if let None = dst {
                    if immediate {
                        dst = Some(
                        if force_dword {
                            DstArg::Imm32(self.read_ip_dword(&mut ip_tmp, &mut next_cycles))
                        } else if ((d == 0 && !size_mismatch) || force_word) && !force_byte {
                            DstArg::Imm16(self.read_ip_word(&mut ip_tmp, &mut next_cycles))
                        } else {
                            DstArg::Imm8(self.read_ip(&mut ip_tmp, &mut next_cycles))
                        })
                    } else {
                        let mod_reg_rm = self.read_ip(&mut ip_tmp, &mut next_cycles);
                        let rm = (mod_reg_rm & 0x07) >> 0;
                        let mod_bits = (mod_reg_rm & 0xC0) >> 6;
                        let arg = self.translate_mod_rm(mod_bits, rm, s, &mut ip_tmp, &mut next_cycles);
                        dst = arg;
                        reg_bits = (mod_reg_rm & 0x38) >> 3;
                    }
                }
            },
            NumArgs::Zero => ()
        }
        (Some(opcode), dst, src, seg, next_cycles, ip_tmp as u16, reg_bits)
    }

    fn get_opcode(&self, code: &u8) -> &Opcode {
        match self.opcodes.get(&code) {
            Some(opcode) => opcode,
            None => match self.opcodes.get(&(code & 0xFE)) {
                Some(opcode) => opcode,
                None => match self.opcodes.get(&(code & 0xFD)) {
                    Some(val) => val,
                    None => self.opcodes.get(&(code & 0xFC)).unwrap()
                }
            }
        }
    }

    fn except(&mut self, code: u8) -> Result<(), String> {
        match code {
            exceptions::DIVIDE_BY_ZERO | exceptions::BOUND | exceptions::INVALID_OPCODE | exceptions::NO_EXTENSION => {
                let opcode_start = self.opcode_address;
                self.regs.get_mut(&Regs::CS).unwrap().value = opcode_start.0;
                self.regs.get_mut(&Regs::IP).unwrap().value = opcode_start.1;
            }
            exceptions::INTO => (),
            _ => return Err(String::from("Invalid exception code!"))
        }

        self.irq = Some(code);
        Ok(())
    }

    fn check_flag(&self, flag: u16) -> bool {
        self.regs[&Regs::FLAGS].value & flag != 0
    }

    fn check_flags_not_equal(&self, flag1: u16, flag2: u16) -> bool {
        self.regs[&Regs::FLAGS].value & flag1 != self.regs[&Regs::FLAGS].value & flag2
    }

    fn read_ip(&self, ip: &mut usize, next_cycles: &mut usize) -> u8 {
        let tmp = *ip;
        *ip += 1;
        *next_cycles += 1;
        let addr = Self::physical_address(self.regs.get(&Regs::CS).unwrap().value, tmp as u16);
        self.ram[addr as usize]
    }

    fn read_ip_mut(&mut self) -> u8 {
        let addr = Self::physical_address(self.regs.get(&Regs::CS).unwrap().value, self.regs.get(&Regs::IP).unwrap().value);
        let val = self.ram[addr as usize];
        self.regs.get_mut(&Regs::IP).unwrap().value += 1;
        self.next_cycles += 1;
        val
    }

    fn get_reg_16(&self, reg_num: u8) -> Option<u16> {
        Some(self.regs.get(&Regs::translate_reg16(reg_num)?)?.value)
    }

    fn get_reg_8(&self, reg_num: u8) -> Option<u8> {
        let (reg, part) = Regs::translate_reg8(reg_num)?;
        match part {
            WordPart::High => Some(self.regs.get(&reg)?.get_high()),
            WordPart::Low => Some(self.regs.get(&reg)?.get_low())
        }
    }

    fn get_src_arg(&self, arg: DstArg, next_cycles: &mut usize) -> Option<SrcArg> {
        match arg {
            DstArg::Reg8(reg) => Some(SrcArg::Byte(self.get_reg_8(reg)?)),
            DstArg::Reg16(reg) => Some(SrcArg::Word(self.get_reg_16(reg)?)),
            DstArg::Imm8(val) => Some(SrcArg::Byte(val)),
            DstArg::Imm16(val) => Some(SrcArg::Word(val)),
            DstArg::Imm32(val) => Some(SrcArg::DWord(val)),
            DstArg::Ptr32(ptr) => { *next_cycles += 4; Some(SrcArg::DWord(self.read_mem_dword(ptr)?)) }
            DstArg::Ptr16(ptr) => { *next_cycles += 2; Some(SrcArg::Word(self.read_mem_word(ptr)?)) },
            DstArg::Ptr8(ptr) => { *next_cycles += 1; Some(SrcArg::Byte(self.read_mem_byte(ptr)?)) },
            DstArg::Reg(reg) => Some(SrcArg::Word(self.regs.get(&reg)?.value))
        }
    }

    fn get_src_arg_mut(&mut self, arg: DstArg) -> Option<SrcArg> {
        match arg {
            DstArg::Reg8(reg) => Some(SrcArg::Byte(self.get_reg_8(reg)?)),
            DstArg::Reg16(reg) => Some(SrcArg::Word(self.get_reg_16(reg)?)),
            DstArg::Imm8(val) => Some(SrcArg::Byte(val)),
            DstArg::Imm16(val) => Some(SrcArg::Word(val)),
            DstArg::Imm32(val) => Some(SrcArg::DWord(val)),
            DstArg::Ptr32(ptr) => Some(SrcArg::DWord(self.read_mem_dword(ptr)?)),
            DstArg::Ptr16(ptr) => Some(SrcArg::Word(self.read_mem_word_mut(ptr)?)),
            DstArg::Ptr8(ptr) => Some(SrcArg::Byte(self.read_mem_byte_mut(ptr)?)),
            DstArg::Reg(reg) => Some(SrcArg::Word(self.regs.get(&reg)?.value))
        }
    }

    fn get_ptr(arg: DstArg) -> Option<u16> {
        match arg {
            DstArg::Ptr32(val) | DstArg::Ptr16(val) | DstArg::Ptr8(val) => Some(val),
            _ => None
        }
    }
    
    fn read_mem_byte(&self, ptr: u16) -> Option<u8> {
        if ptr > self.ram.len() as u16 {
            None
        } else {
            Some(self.ram[Self::physical_address(self.read_reg(self.seg).unwrap(), ptr) as usize])
        }
    }

    fn read_mem_word(&self, ptr: u16) -> Option<u16> {
        Some((self.read_mem_byte(ptr)? as u16) | ((self.read_mem_byte(ptr + 1)? as u16) << 8))
    }

    fn read_mem_dword(&self, ptr: u16) -> Option<u32> {
        Some((self.read_mem_word(ptr)? as u32) | ((self.read_mem_word(ptr + 2)? as u32) << 16))
    }

    fn read_mem_byte_mut(&mut self, ptr: u16) -> Option<u8> {
        if ptr > self.ram.len() as u16 {
            None
        } else {
            self.next_cycles += 1;
            Some(self.ram[Self::physical_address(self.read_reg(self.seg).unwrap(), ptr) as usize])
        }
    }

    fn read_mem_word_mut(&mut self, ptr: u16) -> Option<u16> {
        Some((self.read_mem_byte_mut(ptr)? as u16) | ((self.read_mem_byte_mut(ptr + 1)? as u16) << 8))
    }

    fn read_mem_dword_mut(&mut self, ptr: u16) -> Option<u32> {
        Some((self.read_mem_word_mut(ptr)? as u32) | ((self.read_mem_word_mut(ptr + 2)? as u32) << 16))
    }

    fn write_mem_byte(&mut self, ptr: u16, val: u8) -> Result<(), &str> {
        if ptr > self.ram.len() as u16 {
            Err("Write out of bounds")
        } else {
            let seg_val = self.read_reg(self.seg).unwrap();
            self.ram[Self::physical_address(seg_val, ptr) as usize] = (val & 0xFF) as u8;
            self.next_cycles += 1;
            Ok(())
        }
    }

    fn write_mem_word(&mut self, ptr: u16, val: u16) -> Result<(), &str> {
        self.write_mem_byte(ptr, (val & 0x00FF) as u8).unwrap();
        self.write_mem_byte(ptr + 1, ((val & 0xFF00) >> 8) as u8)
    }

    fn read_mem_byte_seg(&mut self, ptr: u16, seg: Regs) -> Option<u8> {
        if ptr > self.ram.len() as u16 {
            None
        } else {
            self.next_cycles += 1;
            Some(self.ram[Self::physical_address(self.read_reg(seg).unwrap(), ptr) as usize])
        }
    }

    fn read_mem_word_seg(&mut self, ptr: u16, seg: Regs) -> Option<u16> {
        Some((self.read_mem_byte_seg(ptr, seg)? as u16) | ((self.read_mem_byte_seg(ptr + 1, seg)? as u16) << 8))
    }

    fn write_mem_byte_seg(&mut self, ptr: u16, seg: Regs, val: u8) -> Result<(), &str> {
        if ptr > self.ram.len() as u16 {
            Err("Write out of bounds")
        } else {
            let seg_val = self.read_reg(seg).unwrap();
            self.ram[Self::physical_address(seg_val, ptr) as usize] = (val & 0xFF) as u8;
            self.next_cycles += 1;
            Ok(())
        }
    }

    fn write_mem_word_seg(&mut self, ptr: u16, seg: Regs, val: u16) -> Result<(), &str> {
        self.write_mem_byte_seg(ptr, seg, (val & 0x00FF) as u8).unwrap();
        self.write_mem_byte_seg(ptr, seg, ((ptr & 0xFF00) >> 8) as u8)
    }

    fn read_ip_word(&self, ip: &mut usize, next_cycles: &mut usize) -> u16 {
        (self.read_ip(ip, next_cycles) as u16) | ((self.read_ip(ip, next_cycles) as u16) << 8)
    }

    fn read_ip_dword(&self, ip: &mut usize, next_cycles: &mut usize) -> u32 {
        (self.read_ip_word(ip, next_cycles) as u32) | ((self.read_ip_word(ip, next_cycles) as u32) << 16)
    }

    fn read_ip_word_mut(&mut self) -> u16 {
        (self.read_ip_mut() as u16) | ((self.read_ip_mut() as u16) << 8)
    }

    fn write_to_arg(&mut self, arg: DstArg, val_arg: SrcArg) -> Result<(), &str> {
        match arg {
            DstArg::Reg16(reg) => {
                self.regs.get_mut(&Regs::translate_reg16(reg).unwrap()).unwrap().value = if let SrcArg::Word(value) = val_arg {
                    value
                    } else {
                        return Err("Mismatch operand sizes");
                    };
                Ok(())
            },
            DstArg::Reg8(reg_num) => {
                let (reg_enum, part) = Regs::translate_reg8(reg_num).unwrap();
                let reg = self.regs.get_mut(&reg_enum).unwrap();
                let value = if let SrcArg::Byte(val) = val_arg {
                    val
                } else {
                    return Err("Mismatch operand sizes");
                };
                match part {
                    WordPart::Low => { reg.set_low(value) },
                    WordPart::High => { reg.set_high(value) }
                }
                Ok(())
            },
            DstArg::Reg(reg) => {
                self.regs.get_mut(&reg).unwrap().value = match val_arg {
                    SrcArg::Word(val) => val,
                    _ => return Err("Mismatch operand sizes")
                };
                Ok(())
            },
            DstArg::Ptr16(ptr) => {
                match val_arg {
                    SrcArg::Byte(val) => self.write_mem_word(ptr, val as u16),
                    SrcArg::Word(val) => self.write_mem_word(ptr, val),
                    _ => return Err("Mismatch operand sizes")
                }
            },
            DstArg::Ptr8(ptr) => {
                match val_arg {
                    SrcArg::Byte(val) => self.write_mem_byte(ptr, val),
                    SrcArg::Word(val) => self.write_mem_byte(ptr, val as u8),
                    _ => return Err("Mismatch operand sizes")
                }
            },
            _ => Err("Invalid dst arg")
        }
    }

    fn translate_mod_rm(&self, mod_bits: u8, rm: u8, s: u8, ip: &mut usize, next_cycles: &mut usize) -> Option<DstArg> {
        if mod_bits == 0b00 && rm == 0b110 {
            Some(if s == 1{ DstArg::Ptr16(self.read_ip_word(ip, next_cycles)) } else { DstArg::Ptr8(self.read_ip_word(ip, next_cycles)) })
        } else {
            let (reg1, reg2) = match rm {
                0b000 => Some((Regs::BX, Some(Regs::SI))),
                0b001 => Some((Regs::BX, Some(Regs::DI))),
                0b010 => Some((Regs::BP, Some(Regs::SI))),
                0b011 => Some((Regs::BP, Some(Regs::DI))),
                0b100 => Some((Regs::SI, None)),
                0b101 => Some((Regs::DI, None)),
                0b110 => Some((Regs::BP, None)),
                0b111 => Some((Regs::BX, None)),
                _ => None
            }.unwrap();
            let ptr_val = if let Some(reg) = reg2 {
                self.regs.get(&reg1).unwrap().value + self.regs.get(&reg).unwrap().value
            } else {
                self.regs.get(&reg1).unwrap().value
            };
            if s == 2 {
                match mod_bits {
                    0 => Some(DstArg::Ptr32(ptr_val)),
                    1 => Some(DstArg::Ptr32(ptr_val + (self.read_ip(ip, next_cycles) as u16))),
                    2 => Some(DstArg::Ptr32(ptr_val + (self.read_ip_word(ip, next_cycles)))),
                    _ => None
                }
            } else if s == 1 {
                match mod_bits {
                    0 => Some(DstArg::Ptr16(ptr_val)),
                    1 => Some(DstArg::Ptr16(ptr_val + (self.read_ip(ip, next_cycles) as u16))),
                    2 => Some(DstArg::Ptr16(ptr_val + (self.read_ip_word(ip, next_cycles)))),
                    3 => Some(Self::reg_to_arg(rm, s)),
                    _ => None
                }
            } else {
                match mod_bits {
                    0 => Some(DstArg::Ptr8(ptr_val)),
                    1 => Some(DstArg::Ptr8(ptr_val + (self.read_ip(ip, next_cycles) as u16))),
                    2 => Some(DstArg::Ptr8(ptr_val + (self.read_ip_word(ip, next_cycles)))),
                    3 => Some(Self::reg_to_arg(rm, s)),
                    _ => None
                }
            }
        }
    }

    fn translate_placeholder(&self, placeholder: Placeholder, s: u8, ip: &mut usize, next_cycles: &mut usize) -> DstArg {
        match placeholder {
            Placeholder::Reg(reg) => {
                if s == 1 {
                    DstArg::Reg16(reg)
                } else {
                    DstArg::Reg8(reg)
                }
            },
            Placeholder::Imm => {
                if s == 1 {
                    DstArg::Imm16((self.read_ip(ip, next_cycles) as u16) | ((self.read_ip(ip, next_cycles) as u16) << 8))
                } else {
                    DstArg::Imm8(self.read_ip(ip, next_cycles))
                }
            }
            Placeholder::Reg8(reg) => DstArg::Reg8(reg),
            Placeholder::Reg16(reg) => DstArg::Reg16(reg),
            Placeholder::Word(val) => DstArg::Imm16(val),
            Placeholder::Byte(val) => DstArg::Imm8(val),
            Placeholder::Ptr => DstArg::Ptr16((self.read_ip(ip, next_cycles) as u16) | ((self.read_ip(ip, next_cycles) as u16) << 8))
        }
    }

    fn operation_1_arg<T, U>(&mut self, byte: T, word: U) -> SrcArg where
        T: Fn(u8)-> u8,
        U: Fn(u16) -> u16
    {
        match self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Word(dst) => {
                Some(SrcArg::Word(word(dst)))
            },
            SrcArg::Byte(dst) => {
                Some(SrcArg::Byte(byte(dst)))
            },
            _ => None
        }.unwrap()
    }

    fn operation_2_args<T, U>(&mut self, byte: T, word: U) -> SrcArg where
    T: Fn(u8, u8)-> u8,
    U: Fn(u16, u16) -> u16
    {
        match self.src.clone().unwrap() {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    Some(SrcArg::Word(word(src, dst)))
                } else {
                    None
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    Some(SrcArg::Byte(byte(src, dst)))
                } else if let SrcArg::Word(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    Some(SrcArg::Word(word(src as u16, dst)))
                } else {
                    None
                }
            },
            _ => None
        }.unwrap()
    }

    fn sub_command(&mut self, opcode: u8, src: Option<SrcArg>, dst: Option<DstArg>, reg_bits: u8) {
        let tmp_src = self.src.clone();
        let tmp_dst = self.dst.clone();
        let tmp_reg_bits = self.reg_bits;
        self.src = src;
        self.dst = dst;
        self.reg_bits = reg_bits;
        let opcode = self.get_opcode(&opcode).clone();
        self.next_cycles += (opcode.instruction)(self);
        self.src = tmp_src;
        self.dst = tmp_dst;
        self.reg_bits = tmp_reg_bits;
    }

    fn check_carry_add(&mut self, arg: SrcArg) {
        match arg {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(src, dst);
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_8_bit(src, dst);
                } else if let SrcArg::Word(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(src as u16, dst);
                }
            }
            _ => ()
        };
    }

    fn check_flags_in_result(&mut self, result: &SrcArg, flags: u16) {
        if Self::check_flag_in_reg(flags, CPUFlags::AUX_CARRY) {
            self.check_aux_carry(result);
        }
        if Self::check_flag_in_reg(flags, CPUFlags::ZERO) {
            self.check_zero(result);
        }
        if Self::check_flag_in_reg(flags, CPUFlags::SIGN) {
            self.check_sign(result);
        }
    }

    fn check_flag_in_reg(flags: u16, flag: u16) -> bool {
        (flags & flag) > 0
    }

    fn check_zero(&mut self, result: &SrcArg) {
        self.set_flag_if(CPUFlags::ZERO, Self::check_src_arg(result, |val| val == 0, |val| val == 0));
    }

    fn check_aux_carry(&mut self, result: &SrcArg) {
        self.set_flag_if(CPUFlags::AUX_CARRY, Self::check_src_arg(result, |val| (val & 0xF0) != 0, |val| (val & 0xF0) != 0));
    }

    fn check_sign(&mut self, result: &SrcArg) {
        self.set_flag_if(CPUFlags::SIGN, Self::check_src_arg(result, |val| (val & 0x80) != 0, |val| (val & 0x80) != 0));
    }

    fn check_parity(&mut self, result: &SrcArg) {
        self.set_flag_if(CPUFlags::PARITY, Self::check_src_arg(result, |val| (val & 0x01) != 0, |val| (val & 0x01) != 0));
    }

    fn set_flag_if(&mut self, flag: u16, cond: bool) {
        if cond {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    fn set_flag(&mut self, flag: u16) {
        self.regs.get_mut(&Regs::FLAGS).unwrap().value |= flag;
    }

    fn clear_flag(&mut self, flag: u16) {
        self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !flag;
    }

    fn flip_flag(&mut self, flag: u16) {
        self.regs.get_mut(&Regs::FLAGS).unwrap().value ^= flag;
    }

    fn check_src_arg<T, U>(arg: &SrcArg, byte: T, word: U) -> bool where
        T: Fn(u8)-> bool,
        U: Fn(u16) -> bool {
        match arg {
            SrcArg::Byte(val) => byte(*val),
            SrcArg::Word(val) => word(*val),
            _ => panic!("Can't use DWord SrcArg in this opcode")
        }
    }

    fn check_carry_sub(&mut self, arg: SrcArg) {
        match arg {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(dst, alu::twos_compliment_word(src));
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_8_bit(dst, alu::twos_compliment_byte(src));
                } else if let SrcArg::Word(dst) = self.get_src_arg_mut(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(dst, alu::twos_compliment_word(src as u16));
                }
            }
            _ => ()
        };
    }

    fn check_carry_16_bit(&mut self, arg1: u16, arg2: u16) {
        if (arg1 as u32) + (arg2 as u32) > 65535 {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::CARRY | CPUFlags::OVERFLOW;
        } else {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !(CPUFlags::CARRY | CPUFlags::OVERFLOW);
        }
    }

    fn check_carry_8_bit(&mut self, arg1: u8, arg2: u8) {
        if (arg1 as u16) + (arg2 as u16) > 255 {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::CARRY;
        } else {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !CPUFlags::CARRY;
        }
    }

    fn reg_to_arg(reg: u8, s: u8) -> DstArg {
        if s == 1 {
            DstArg::Reg16(reg)
        } else {
            DstArg::Reg8(reg)
        }
    }

    pub fn read_reg(&self, reg: Regs) -> Option<u16> {
        match self.regs.get(&reg) {
            Some(val) => Some(val.value),
            None => None
        }
    }

    pub fn read_mem(&self, loc: usize) -> u8 {
        self.ram[loc]
    }

    pub fn load(&mut self, data: Vec<u8>, loc: usize) {
        for i in 0..data.len() {
            self.ram[loc + i] = data[i];
        }
    }

    pub fn execute_next(&mut self) {
        self.step();
        while match self.instruction { Some(_) => true, None => false } || self.next_cycles > 0 {
            self.step();
        }
    }

    pub fn execute_next_from(&mut self, loc: u16) {
        self.regs.get_mut(&Regs::IP).unwrap().value = loc;
        self.execute_next();
    }

    pub fn run_to_nop(&mut self, loc: u16) {
        self.regs.get_mut(&Regs::IP).unwrap().value = loc;
        self.step();
        while match self.instruction.clone() { Some(instruction) => !instruction.has_flag(OpcodeFlags::Nop.into()), None => true } {
            self.step();
        }
    }

    pub fn set_reg(&mut self, reg: Regs, val: u16) {
        self.regs.get_mut(&reg).unwrap().value = val
    }

    pub fn get_mem_seg(&self, seg: Regs, loc: u16) -> u8 {
        let seg_val = self.read_reg(seg).unwrap();
        self.ram[Self::physical_address(seg_val, loc) as usize]
    }

    pub fn get_instruction_text(&self, loc: usize) -> Option<String> {
        let (opcode, dst, src, _, _, _, reg_bits) = self.decode_instruction(loc);
        let mnemonic = opcode.clone()?.mnemonic;
        Some(match opcode?.clone().num_args {
            NumArgs::Zero => (mnemonic)(reg_bits)?,
            NumArgs::One => format!("{} {}", (mnemonic)(reg_bits)?, dst?.to_text()),
            NumArgs::Two => format!("{} {}, {}", (mnemonic)(reg_bits)?, dst?.to_text(), src?.to_text())
        })
    }

    fn physical_address(seg: u16, offset: u16) -> u32 {
        ((seg as u32) << 4) + (offset as u32)
    }
}
