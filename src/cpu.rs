mod reg;
mod mem;

use std::collections::HashMap;
use std::rc::Rc;
use std::option::Option::Some;

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
    Ptr(u16)
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

#[derive(Clone, Copy, Debug)]
pub enum AddressingMode {
    Immediate,
    Direct,
    Indirect,
    IndirectIndex,
    Relative,
    SIB
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
    IP
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
    immediate: bool
}

impl Opcode {
    fn new(instruction: Rc<dyn Fn(&mut CPU) -> usize>, num_args: NumArgs, cycles: usize, shorthand: Option<(Placeholder, Option<Placeholder>)>, immediate: bool) -> Self {
        Self {
            instruction,
            num_args,
            cycles,
            shorthand,
            immediate
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
    next_cycles: usize,
}

impl CPU {
    pub fn new(ram_size: usize) -> Self {
        // Create and allocate the virtual ram
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

        // Define opcodes
        let mut opcodes: HashMap<u8, Opcode> = HashMap::new();
        // Move opcodes
        opcodes.insert(0x88, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, None, false));
        opcodes.insert(0xA0, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Ptr))), false));
        for x in 0..7 {
            opcodes.insert(0xB0 + x, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, Some((Placeholder::Reg8(x), Some(Placeholder::Imm))), true));
            opcodes.insert(0xB8 + x, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, Some((Placeholder::Reg16(x), Some(Placeholder::Imm))), true));
        }
        opcodes.insert(0xC6, Opcode::new(Rc::new(Self::mov), NumArgs::Two, 1, None, true));

        Self {
            ram,
            regs,
            opcodes,
            instruction: None,
            src: None,
            dst: None,
            next_cycles: 0,
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
            let code = self.read_ip();
            let d = (code & 0x02) >> 1;
            let mut s = (code & 0x01) >> 0;
            let opcode = match self.opcodes.get(&code) {
                Some(opcode) => opcode,
                None => self.opcodes.get(&(code & 0xFC)).unwrap()
            };
            self.instruction = Some(opcode.clone());
            self.next_cycles += opcode.cycles;
            let immediate = opcode.immediate;
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
                if d == 1 && !immediate {
                    self.src = self.get_src_arg(arg1_translated.unwrap(), s);
                    self.dst = arg2_translated;
                } else {
                    self.src = self.get_src_arg(arg2_translated.unwrap(), s);
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
                                if s == 1 {
                                    DstArg::Imm16(self.read_ip_word())
                                } else {
                                    DstArg::Imm8(self.read_ip())
                                }
                            )
                        } else {
                            Some(Self::reg_to_arg(reg, s))
                        };

                        if d == 1 || immediate {
                            if let None = self.src {
                                self.src = self.get_src_arg(arg1.unwrap(), s);
                            }
                            if let None = self.dst {
                                self.dst = arg2;
                            }
                        } else {
                            if let None = self.src {
                                self.src = self.get_src_arg(arg2.unwrap(), s);
                            }
                            if let None = self.dst {
                                self.dst = arg1;
                            }
                        }
                    }
                },
                NumArgs::One => {
                    let mod_reg_rm = self.read_ip();
                    let rm = (mod_reg_rm & 0x07) >> 0;
                    let mod_bits = (mod_reg_rm & 0xC0) >> 6;
                    let arg = self.translate_mod_rm(mod_bits, rm, s);
                    self.src = self.get_src_arg(arg.unwrap(), s);
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

    fn get_src_arg(&mut self, arg: DstArg, s: u8) -> Option<SrcArg> {
        match arg {
            DstArg::Reg8(reg) => Some(SrcArg::Byte(self.get_reg_8(reg)?)),
            DstArg::Reg16(reg) => Some(SrcArg::Word(self.get_reg_16(reg)?)),
            DstArg::Imm8(val) => Some(SrcArg::Byte(val)),
            DstArg::Imm16(val) => Some(SrcArg::Word(val)),
            DstArg::Ptr(ptr) => if s == 1 { Some(SrcArg::Word(self.read_mem_word(ptr)?)) } else { Some(SrcArg::Byte(self.read_mem_byte(ptr)?)) }
        }
    }

    fn read_mem_byte(&mut self, ptr: u16) -> Option<u8> {
        if ptr > self.ram.len() as u16 {
            None
        } else {
            self.next_cycles += 1;
            Some(self.ram[ptr as usize])
        }
    }

    fn read_mem_word(&mut self, ptr: u16) -> Option<u16> {
        Some((self.read_mem_byte(ptr)? as u16) | ((self.read_mem_byte(ptr + 1)? as u16) << 8))
    }

    fn write_mem_byte(&mut self, ptr: u16, val: u8) -> Result<(), &str> {
        if ptr > self.ram.len() as u16 {
            Err("Write out of bounds")
        } else {
            self.ram[ptr as usize] = (val & 0xFF) as u8;
            self.next_cycles += 1;
            Ok(())
        }
    }

    fn write_mem_word(&mut self, ptr: u16, val: u16) -> Result<(), &str> {
        self.write_mem_byte(ptr, (val & 0x00FF) as u8);
        self.write_mem_byte(ptr, ((ptr & 0xFF00) >> 8) as u8)
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
                        return Err("Mismatch oparend sizes");
                    };
                Ok(())
            },
            DstArg::Reg8(reg_num) => {
                let (reg_enum, part) = Self::translate_reg8(reg_num).unwrap();
                let reg = self.regs.get_mut(&reg_enum).unwrap();
                let value = if let SrcArg::Byte(val) = val_arg {
                    val
                } else {
                    return Err("Mismatch oparend sizes");
                };
                match part {
                    WordPart::Low => { reg.set_low(value) },
                    WordPart::High => { reg.set_high(value) }
                }
                Ok(())
            },
            DstArg::Ptr(ptr) => {
                match val_arg {
                    SrcArg::Byte(val) => self.write_mem_byte(ptr, val),
                    SrcArg::Word(val) => self.write_mem_word(ptr, val)
                }
            },
            _ => Err("Invalid dst arg")
        }
    }

    fn translate_mod_rm(&mut self, mod_bits: u8, rm: u8, s: u8) -> Option<DstArg> {
        if mod_bits == 0b00 && rm == 0b101 {
            Some(DstArg::Ptr(self.read_ip_word()))
        } else if rm == 0b100 {
            self.translate_sib(mod_bits)
        } else {
            match mod_bits {
                0 => Some(DstArg::Ptr(self.regs.get(&Self::translate_reg16(rm).unwrap()).unwrap().value)),
                1 => Some(DstArg::Ptr(self.regs.get(&Self::translate_reg16(rm).unwrap()).unwrap().value + (self.read_ip() as u16))),
                2 => Some(DstArg::Ptr(self.regs.get(&Self::translate_reg16(rm).unwrap()).unwrap().value + (self.read_ip_word()))),
                3 => Some(Self::reg_to_arg(rm, s)),
                _ => None
            }
        }
    }

    fn translate_sib(&mut self, mod_bits: u8) -> Option<DstArg> {
        let sib = self.read_ip();
        let displacement = match mod_bits {
            1 => (self.read_ip() as i8) as i16,
            2 => self.read_ip_word() as i16,
            _ => 0
        };
        let scale_bits = (sib & 0xC0) >> 6;
        let index_bits = (sib & 0x38) >> 3;
        let base_bits = (sib & 0x07) >> 0;
        let scale_value = if scale_bits < 4 { 2_i16.pow(scale_bits as u32) } else { 0 };
        let index_value = self.regs.get(&(if index_bits == 0b100 { None } else { Self::translate_reg16(index_bits) }).unwrap()).unwrap().value as i16;
        let base_value = if mod_bits == 0 && base_bits == 0b101 { 0 } else { self.regs.get(&Self::translate_reg16(base_bits).unwrap()).unwrap().value } as i16;
        Some(DstArg::Ptr(((index_value * scale_value) + base_value + displacement) as u16))
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
            Placeholder::Ptr => DstArg::Ptr((self.read_ip() as u16) | ((self.read_ip() as u16) << 8))
        }
    }

    fn get_reg_high(&self, num: u8) -> u8 {
        let reg = self.regs.get(&Self::translate_reg16(num % 4).unwrap()).unwrap();
        reg.get_high()
    }

    fn get_reg_low(&self, num: u8) -> u8 {
        let reg = self.regs.get(&Self::translate_reg16(num % 4).unwrap()).unwrap();
        reg.get_low()
    }

    fn set_reg_high(&mut self, num: u8, val: u8) {
        let reg = self.regs.get_mut(&Self::translate_reg16(num % 4).unwrap()).unwrap();
        reg.set_high(val);
    }

    fn set_reg_low(&mut self, num: u8, val: u8) {
        let reg = self.regs.get_mut(&Self::translate_reg16(num % 4).unwrap()).unwrap();
        reg.set_low(val);
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
}
