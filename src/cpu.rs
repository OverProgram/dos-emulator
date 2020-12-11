mod reg;
mod mem;
mod alu;
mod stack;

use std::collections::HashMap;
use std::rc::Rc;

struct OpcodeFlags;

impl OpcodeFlags {
    const NONE: u32 = 0;
    const IMMEDIATE: u32 = 0x01;
    const SIZE_MISMATCH: u32 = 0x02;
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

#[derive(Clone, Copy, Debug)]
enum NumArgs {
    Zero,
    One,
    Two
}

#[derive(Clone, Debug)]
enum DstArg {
    Reg8(u8),
    Reg16(u8),
    Imm8(u8),
    Imm16(u16),
    Ptr16(u16),
    Ptr8(u16)
}

#[derive(Clone, Debug)]
enum SrcArg {
    Byte(u8),
    Word(u16)
}

#[derive(Clone, Debug)]
enum Placeholder {
    Reg8(u8),
    Reg16(u8),
    Reg(u8),
    Imm,
    Ptr
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
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
    flags: u32,
    segment: Regs
}

impl Opcode {
    fn new(instruction: Rc<dyn Fn(&mut CPU) -> usize>, num_args: NumArgs, cycles: usize, shorthand: Option<(Placeholder, Option<Placeholder>)>, segment: Regs, flags: u32) -> Self {
        Self {
            instruction,
            num_args,
            cycles,
            shorthand,
            flags,
            segment
        }
    }

    fn has_flag(&self, flag: u32) -> Result<bool, &str> {
        match flag {
            OpcodeFlags::SIZE_MISMATCH | OpcodeFlags::IMMEDIATE => Ok(if (self.flags & flag) > 0 { true } else { false }),
            _ => Err("invalid flags!")
        }
    }
}

pub struct CPU {
    ram: Vec<u8>,
    regs: HashMap<Regs, reg::Reg>,
    opcodes: HashMap<u8, Opcode>,
    instruction: Option<Opcode>,
    src: Option<SrcArg>,
    dst: Option<DstArg>,
    seg: Regs,
    next_cycles: usize,
    reg_bits: u8
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
        // Move opcodes
        opcodes.insert(0x88, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::NONE));
        opcodes.insert(0xA0, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Ptr))), Regs::DS, OpcodeFlags::NONE));
        for x in 0..7 {
            opcodes.insert(0xB0 + x, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, Some((Placeholder::Reg8(x), Some(Placeholder::Imm))),Regs::DS, OpcodeFlags::IMMEDIATE));
            opcodes.insert(0xB8 + x, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, Some((Placeholder::Reg16(x), Some(Placeholder::Imm))), Regs::DS, OpcodeFlags::IMMEDIATE));
        }
        opcodes.insert(0xC6, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::IMMEDIATE));
        // ALU opcodes
        let mut alu_opcodes: Vec<(Rc<dyn Fn(&mut CPU) -> usize>, u8)> = Vec::new();
        alu_opcodes.push((Rc::new(Self::add), 0x00));
        alu_opcodes.push((Rc::new(Self::sub), 0x28));
        alu_opcodes.push((Rc::new(Self::xor), 0x30));
        alu_opcodes.push((Rc::new(Self::and), 0x20));
        alu_opcodes.push((Rc::new(Self::or), 0x08));
        for (instruction, offset) in alu_opcodes.into_iter() {
            opcodes.insert(0x00 + offset, Opcode::new(instruction.clone(), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::NONE));
            opcodes.insert(0x04 + offset, Opcode::new(instruction.clone(), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Imm))), Regs::DS, OpcodeFlags::IMMEDIATE));
        }
        opcodes.insert(0x80, Opcode::new(Rc::new(Self::alu_dispatch_two_args), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::IMMEDIATE));
        for x in 0..7 {
            opcodes.insert(0x40 + x, Opcode::new(Rc::new(Self::inc), NumArgs::Zero, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, OpcodeFlags::NONE));
            opcodes.insert(0x48 + x, Opcode::new(Rc::new(Self::inc), NumArgs::Zero, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, OpcodeFlags::NONE));
        }
        opcodes.insert(0x83, Opcode::new(Rc::new(Self::alu_dispatch_two_args), NumArgs::Two, 1, None, Regs::DS, OpcodeFlags::IMMEDIATE | OpcodeFlags::SIZE_MISMATCH));
        opcodes.insert(0xFE, Opcode::new(Rc::new(Self::alu_dispatch_one_arg), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::NONE));
        opcodes.insert(0xF6, Opcode::new(Rc::new(Self::mul_dispatch), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::NONE));
        // Stack opcodes
        for x in 0..7 {
            opcodes.insert(0x50 + x, Opcode::new(Rc::new(Self::push), NumArgs::One, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, OpcodeFlags::NONE));
            opcodes.insert(0x58 + x, Opcode::new(Rc::new(Self::pop), NumArgs::One, 1, Some((Placeholder::Reg16(x), None)), Regs::DS, OpcodeFlags::NONE));
        }
        opcodes.insert(0x8F, Opcode::new(Rc::new(Self::pop), NumArgs::One, 1, None, Regs::DS, OpcodeFlags::NONE));

        Self {
            ram,
            regs,
            opcodes,
            instruction: None,
            src: None,
            dst: None,
            seg: Regs::DS,
            next_cycles: 0,
            reg_bits: 0
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
        } else {
            let mut seg: Option<Regs> = None;
            let code = self.read_ip();
            let d = (code & 0x02) >> 1;
            let mut s = (code & 0x01) >> 0;
            let opcode = match self.opcodes.get(&code) {
                Some(opcode) => opcode,
                None => match self.opcodes.get(&(code & 0xFE)) {
                    Some(opcode) => opcode,
                    None => self.opcodes.get(&(code & 0xFC)).unwrap()
                }
            };
            self.instruction = Some(opcode.clone());
            self.next_cycles += opcode.cycles;
            self.seg = if let None = seg {
                opcode.segment
            } else {
                seg.unwrap()
            };
            let immediate = opcode.has_flag(OpcodeFlags::IMMEDIATE).unwrap();
            let size_mismatch = opcode.has_flag(OpcodeFlags::SIZE_MISMATCH).unwrap();
            let num_args = opcode.num_args;
            let shorthand = opcode.shorthand.clone();

            if let Some((arg1, arg2)) = opcode.shorthand.clone() {
                let arg1_translated = Some(self.translate_placeholder(arg1, s));
                s = match arg1_translated.clone().unwrap() {
                    DstArg::Reg8(_) => 0,
                    DstArg::Reg16(_) => 1,
                    _ => s
                };
                let mut arg2_translated = None;
                if let Some(arg) = arg2 {
                    arg2_translated = Some(self.translate_placeholder(arg, s));
                }
                let one_arg = if let NumArgs::Two = num_args { false } else { true };
                if d == 1 && !immediate && !one_arg {
                    self.src = match arg1_translated {
                        Some(arg) => self.get_src_arg(arg),
                        None => None
                    };
                    self.dst = arg2_translated;
                } else {
                    self.src = match arg2_translated {
                        Some(arg) => self.get_src_arg(arg),
                        None => None
                    };
                    self.dst = arg1_translated;
                }
            }
            match num_args {
                NumArgs::Two => {
                    if let None = shorthand {
                        let mod_reg_rm = self.read_ip();
                        let rm = (mod_reg_rm & 0x07) >> 0;
                        let reg = (mod_reg_rm & 0x38) >> 3;
                        let mod_bits = (mod_reg_rm & 0xC0) >> 6;
                        let arg2 = {
                            if let None = self.src {
                                self.translate_mod_rm(mod_bits, rm, s)
                            } else {
                                None
                            }
                        };
                        let arg1 = if immediate {
                            Some(
                                if s == 1 && !size_mismatch {
                                    DstArg::Imm16(self.read_ip_word())
                                } else {
                                    DstArg::Imm8(self.read_ip())
                                }
                            )
                        } else {
                            Some(Self::reg_to_arg(reg, s))
                        };

                        if d == 0 || immediate {
                            if let None = self.src {
                                self.src = self.get_src_arg(arg1.unwrap());
                            }
                            if let None = self.dst {
                                self.dst = arg2;
                            }
                        } else {
                            if let None = self.src {
                                self.src = self.get_src_arg(arg2.unwrap());
                            }
                            if let None = self.dst {
                                self.dst = arg1;
                            }
                        }

                        self.reg_bits = reg;
                    }
                },
                NumArgs::One => {
                    let mod_reg_rm = self.read_ip();
                    let rm = (mod_reg_rm & 0x07) >> 0;
                    let mod_bits = (mod_reg_rm & 0xC0) >> 6;
                    let arg = self.translate_mod_rm(mod_bits, rm, s);
                    self.dst = arg;
                    self.reg_bits = (mod_reg_rm & 0x38) >> 3;
                },
                NumArgs::Zero => ()
            }
        }
    }

    fn read_ip(&mut self) -> u8 {
        let val = self.ram[self.regs.get(&Regs::IP).unwrap().value as usize];
        self.regs.get_mut(&Regs::IP).unwrap().value += 1;
        self.next_cycles += 1;
        val
    }

    fn get_reg_16(&self, reg_num: u8) -> Option<u16> {
        Some(self.regs.get(&Self::translate_reg16(reg_num)?)?.value)
    }

    fn get_reg_8(&self, reg_num: u8) -> Option<u8> {
        let (reg, part) = Self::translate_reg8(reg_num)?;
        match part {
            WordPart::High => Some(self.regs.get(&reg)?.get_high()),
            WordPart::Low => Some(self.regs.get(&reg)?.get_low())
        }
    }

    fn get_src_arg(&mut self, arg: DstArg) -> Option<SrcArg> {
        match arg {
            DstArg::Reg8(reg) => Some(SrcArg::Byte(self.get_reg_8(reg)?)),
            DstArg::Reg16(reg) => Some(SrcArg::Word(self.get_reg_16(reg)?)),
            DstArg::Imm8(val) => Some(SrcArg::Byte(val)),
            DstArg::Imm16(val) => Some(SrcArg::Word(val)),
            DstArg::Ptr16(ptr) => Some(SrcArg::Word(self.read_mem_word(ptr)?)),
            DstArg::Ptr8(ptr) => Some(SrcArg::Byte(self.read_mem_byte(ptr)?))
        }
    }

    fn read_mem_byte(&mut self, ptr: u16) -> Option<u8> {
        if ptr > self.ram.len() as u16 {
            None
        } else {
            self.next_cycles += 1;
            Some(self.ram[Self::physical_address(self.read_reg(self.seg).unwrap(), ptr) as usize])
        }
    }

    fn read_mem_word(&mut self, ptr: u16) -> Option<u16> {
        Some((self.read_mem_byte(ptr)? as u16) | ((self.read_mem_byte(ptr + 1)? as u16) << 8))
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
        self.write_mem_byte(ptr, ((ptr & 0xFF00) >> 8) as u8)
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

    fn read_ip_word(&mut self) -> u16 {
        (self.read_ip() as u16) | ((self.read_ip() as u16) << 8)
    }

    fn write_to_arg(&mut self, arg: DstArg, val_arg: SrcArg) -> Result<(), &str> {
        match arg {
            DstArg::Reg16(reg) => {
                self.regs.get_mut(&Self::translate_reg16(reg).unwrap()).unwrap().value = if let SrcArg::Word(value) = val_arg {
                        value
                    } else {
                        return Err("Mismatch operand sizes");
                    };
                Ok(())
            },
            DstArg::Reg8(reg_num) => {
                let (reg_enum, part) = Self::translate_reg8(reg_num).unwrap();
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
            DstArg::Ptr16(ptr) => {
                match val_arg {
                    SrcArg::Byte(val) => self.write_mem_word(ptr, val as u16),
                    SrcArg::Word(val) => self.write_mem_word(ptr, val)
                }
            },
            DstArg::Ptr8(ptr) => {
                match val_arg {
                    SrcArg::Byte(val) => self.write_mem_byte(ptr, val),
                    SrcArg::Word(val) => self.write_mem_byte(ptr, val as u8)
                }
            },
            _ => Err("Invalid dst arg")
        }
    }

    fn translate_mod_rm(&mut self, mod_bits: u8, rm: u8, s: u8) -> Option<DstArg> {
        if mod_bits == 0b00 && rm == 0b110 {
            Some(if s == 1{ DstArg::Ptr16(self.read_ip_word()) } else { DstArg::Ptr8(self.read_ip_word()) })
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
            if s == 1 {
                match mod_bits {
                    0 => Some(DstArg::Ptr16(ptr_val)),
                    1 => Some(DstArg::Ptr16(ptr_val + (self.read_ip() as u16))),
                    2 => Some(DstArg::Ptr16(ptr_val + (self.read_ip_word()))),
                    3 => Some(Self::reg_to_arg(rm, s)),
                    _ => None
                }
            } else {
                match mod_bits {
                    0 => Some(DstArg::Ptr8(ptr_val)),
                    1 => Some(DstArg::Ptr8(ptr_val + (self.read_ip() as u16))),
                    2 => Some(DstArg::Ptr8(ptr_val + (self.read_ip_word()))),
                    3 => Some(Self::reg_to_arg(rm, s)),
                    _ => None
                }
            }
        }
    }

    fn translate_placeholder(&mut self, placeholder: Placeholder, s: u8) -> DstArg {
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
                    DstArg::Imm16((self.read_ip() as u16) | ((self.read_ip() as u16) << 8))
                } else {
                    DstArg::Imm8(self.read_ip())
                }
            }
            Placeholder::Reg8(reg) => DstArg::Reg8(reg),
            Placeholder::Reg16(reg) => DstArg::Reg16(reg),
            Placeholder::Ptr => DstArg::Ptr16((self.read_ip() as u16) | ((self.read_ip() as u16) << 8))
        }
    }

    fn operation_1_arg<T, U>(&mut self, byte: T, word: U) -> SrcArg where
        T: Fn(u8)-> u8,
        U: Fn(u16) -> u16
    {
        match self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Word(dst) => {
                Some(SrcArg::Word(word(dst)))
            },
            SrcArg::Byte(dst) => {
                Some(SrcArg::Byte(byte(dst)))
            }
        }.unwrap()
    }

    fn operation_2_args<T, U>(&mut self, byte: T, word: U) -> SrcArg where
    T: Fn(u8, u8)-> u8,
    U: Fn(u16, u16) -> u16
    {
        match self.src.clone().unwrap() {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    Some(SrcArg::Word(word(src, dst)))
                } else {
                    None
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    Some(SrcArg::Byte(byte(src, dst)))
                } else if let SrcArg::Word(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    Some(SrcArg::Word(word(src as u16, dst)))
                } else {
                    None
                }
            }
        }.unwrap()
    }

    fn check_carry_add(&mut self, arg: SrcArg) {
        match arg {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(src, dst);
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_8_bit(src, dst);
                } else if let SrcArg::Word(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(src as u16, dst);
                }
            }
        };
    }

    fn check_zero(&mut self, result: &SrcArg) {
        if Self::check_src_arg(result, 0) {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::ZERO;
        } else {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !CPUFlags::ZERO;
        }
    }

    fn check_src_arg(arg: &SrcArg, eq: u16) -> bool {
        match arg {
            SrcArg::Byte(val) => *val == eq as u8,
            SrcArg::Word(val) => *val == eq
        }
    }

    fn twos_compliment_word(arg: u16) -> u16 {
        Self::add_with_carry_16_bit(!arg, 1)
    }

    fn twos_compliment_byte(arg: u8) -> u8 {
        Self::add_with_carry_8_bit(!arg, 1)
    }

    fn check_carry_sub(&mut self, arg: SrcArg) {
        match arg {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(dst, Self::twos_compliment_word(src));
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_8_bit(dst, Self::twos_compliment_byte(src));
                } else if let SrcArg::Word(dst) = self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
                    self.check_carry_16_bit(dst, Self::twos_compliment_word(src as u16));
                }
            }
        };
    }

    fn check_carry_16_bit(&mut self, arg1: u16, arg2: u16) {
        if (arg1 as u32) + (arg2 as u32) > 65535 {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::CARRY;
        } else {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !CPUFlags::CARRY;
        }
    }

    fn check_carry_8_bit(&mut self, arg1: u8, arg2: u8) {
        if (arg1 as u16) + (arg2 as u16) > 255 {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::CARRY;
        } else {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !CPUFlags::CARRY;
        }
    }

    fn add_with_carry_16_bit(arg1: u16, arg2: u16) -> u16 {
        let sum = ((arg1 as u32) + (arg2 as u32)) % 65536;
        sum as u16
    }

    fn add_with_carry_8_bit(arg1: u8, arg2: u8) -> u8 {
        let sum = ((arg1 as u16) + (arg2 as u16)) % 256;
        sum as u8
    }

    fn sub_with_carry_16_bit(arg1: u16, arg2: u16) -> u16 {
        let mut sum = (arg1 as i32) - (arg2 as i32);
        if sum < 0 {
            sum += 65536;
        }
        sum as u16
    }

    fn sub_with_carry_8_bit( arg1: u8, arg2: u8) -> u8 {
        let mut sum = (arg1 as i16) - (arg2 as i16);
        if sum < 0 {
            sum += 256;
        }
        sum as u8
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
        self.regs.get_mut(&Regs::IP).unwrap().value = loc as u16;
    }

    pub fn execute_next(&mut self) {
        self.step();
        while match self.instruction { Some(_) => true, None => false } {
            self.step();
        }
    }

    pub fn execute_next_from(&mut self, loc: u16) {
        self.regs.get_mut(&Regs::IP).unwrap().value = loc;
        self.execute_next();
    }

    pub fn set_reg(&mut self, reg: Regs, val: u16) {
        self.regs.get_mut(&reg).unwrap().value = val
    }

    pub fn get_mem_seg(&self, seg: Regs, loc: u16) -> u8 {
        let seg_val = self.read_reg(seg).unwrap();
        self.ram[Self::physical_address(seg_val, loc) as usize]
    }

    fn translate_reg16(num: u8) -> Option<Regs> {
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

    fn translate_reg8(num: u8) -> Option<(Regs, WordPart)> {
        Some((Self::translate_reg16(num % 4)?, if num / 2 == 0 { WordPart::Low } else { WordPart::High }))
    }

    fn physical_address(seg: u16, offset: u16) -> u32 {
        ((seg << 4) as u32) + (offset as u32)
    }
}
