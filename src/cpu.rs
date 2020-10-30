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
enum Arg {
    Reg8(u8),
    Reg16(u8),
    Imm8(u8),
    Imm16(u16),
    Ptr(u16)
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

pub enum Regs8 {
    AH,
    BH,
    CH,
    DH,
    AL,
    BL,
    CL,
    DL
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
    src: Option<Arg>,
    dst: Option<Arg>,
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
        opcodes.insert(0x88, Opcode::new(Rc::new(Self::mov_reg), NumArgs::Two, 1, None, false));
        // opcodes.insert(0xA0, Opcode::new(Rc::new(Self::mov_ax), One, 1, None, false));
        opcodes.insert(0xA0, Opcode::new(Rc::new(Self::mov_reg), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Ptr))), false));
        for x in 0..7 {
            opcodes.insert(0xB0 + x, Opcode::new(Rc::new(Self::mov_imm), NumArgs::Two, 1, Some((Placeholder::Reg8(x), Some(Placeholder::Imm))), true));
            opcodes.insert(0xB8 + x, Opcode::new(Rc::new(Self::mov_imm), NumArgs::Two, 1, Some((Placeholder::Reg16(x), Some(Placeholder::Imm))), true));
        }
        opcodes.insert(0xC6, Opcode::new(Rc::new(Self::mov_imm), NumArgs::Two, 1, None, true));

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
                    Arg::Reg8(_) => 0,
                    Arg::Reg16(_) => 1,
                    _ => s
                };
                let mut arg2_translated = None;
                if let Some(arg) = arg2 {
                    arg2_translated = Some(self.translate_placeholder(arg, s));
                }
                if d == 1 && !immediate {
                    self.src = arg1_translated;
                    self.dst = arg2_translated;
                } else {
                    self.src = arg2_translated;
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
                                    Arg::Imm16(self.read_ip_word())
                                } else {
                                    Arg::Imm8(self.read_ip())
                                }
                            )
                        } else {
                            Some(Self::reg_to_arg(reg, s))
                        };

                        if d == 1 || immediate {
                            if let None = self.src {
                                self.src = arg1;
                            }
                            if let None = self.dst {
                                self.dst = arg2;
                            }
                        } else {
                            if let None = self.src {
                                self.src = arg2;
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
                    self.src = self.translate_mod_rm(mod_bits, rm, s);
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

    fn read_ip_word(&mut self) -> u16 {
        (self.read_ip() as u16) | ((self.read_ip() as u16) << 8)
    }

    fn translate_mod_rm(&mut self, mod_bits: u8, rm: u8, s: u8) -> Option<Arg> {
        if mod_bits == 0b00 && rm == 0b101 {
            Some(Arg::Ptr(self.read_ip_word()))
        } else if rm == 0b100 {
            self.translate_sib(mod_bits, rm)
        } else {
            match mod_bits {
                0 => Some(Arg::Ptr(self.regs.get(&Self::translate_reg16(rm).unwrap()).unwrap().value)),
                1 => Some(Arg::Ptr(self.regs.get(&Self::translate_reg16(rm).unwrap()).unwrap().value + (self.read_ip() as u16))),
                2 => Some(Arg::Ptr(self.regs.get(&Self::translate_reg16(rm).unwrap()).unwrap().value + (self.read_ip_word()))),
                3 => Some(Self::reg_to_arg(rm, s)),
                _ => None
            }
        }
    }

    fn translate_sib(&mut self, mod_bits: u8, rm: u8) -> Option<Arg> {
        let sib = self.read_ip();
        // let displacement = (self.read_ip() as i8) as i16;
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
        Some(Arg::Ptr(((index_value * scale_value) + base_value + displacement) as u16))
    }

    fn translate_placeholder(&mut self, placeholder: Placeholder, s: u8) -> Arg {
        match placeholder {
            Placeholder::Reg(reg) => {
                if s == 1 {
                    Arg::Reg16(reg)
                } else {
                    Arg::Reg8(reg)
                }
            },
            Placeholder::Imm => {
                if s == 1 {
                    Arg::Imm16((self.read_ip() as u16) | ((self.read_ip() as u16) << 8))
                } else {
                    Arg::Imm8(self.read_ip())
                }
            }
            Placeholder::Reg8(reg) => Arg::Reg8(reg),
            Placeholder::Reg16(reg) => Arg::Reg16(reg),
            Placeholder::Ptr => Arg::Ptr((self.read_ip() as u16) | ((self.read_ip() as u16) << 8))
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

    fn reg_to_arg(reg: u8, s: u8) -> Arg {
        if s == 1 {
            Arg::Reg16(reg)
        } else {
            Arg::Reg8(reg)
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

    fn translate_reg8(num: u8) -> Option<Regs> {
        Self::translate_reg16(num % 4)
    }
}
