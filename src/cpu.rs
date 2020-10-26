mod reg;
mod mem;

use std::collections::HashMap;
use std::rc::Rc;
use std::option::Option::Some;

#[derive(Clone, Copy)]
enum NumArgs {
    Zero,
    One,
    Two
}

#[derive(Clone)]
enum Arg {
    Reg8(u8),
    Reg16(u8),
    Imm8(u8),
    Imm16(u16),
    Ptr(u16)
}

#[derive(Clone)]
enum Placeholder {
    Reg8(u8),
    Reg16(u8),
    Reg(u8),
    Ptr
}

#[derive(Clone, Copy)]
pub enum AddressingMode {
    Immediate,
    Direct,
    Indirect,
    IndirectIndex,
    Relative,
    SIB
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

struct CPU {
    ram: Vec<u8>,
    regs: HashMap<String, reg::Reg>,
    opcodes: HashMap<u8, Opcode>,
    instruction: Option<Opcode>,
    src: Option<Arg>,
    dst: Option<Arg>,
    next_cycles: usize,
}

impl CPU {
    pub fn new(ram_size: usize) -> Self {
        // Create and allocate the virtual ram
        let ram: Vec<u8> = Vec::with_capacity(ram_size);

        // Create register HashMap
        let mut regs: HashMap<String, reg::Reg> = HashMap::new();
        regs.insert(String::from("ax"), reg::Reg::new());
        regs.insert(String::from("bx"), reg::Reg::new());
        regs.insert(String::from("cx"), reg::Reg::new());
        regs.insert(String::from("dx"), reg::Reg::new());
        regs.insert(String::from("si"), reg::Reg::new());
        regs.insert(String::from("di"), reg::Reg::new());
        regs.insert(String::from("bp"), reg::Reg::new());
        regs.insert(String::from("sp"), reg::Reg::new());
        regs.insert(String::from("es"), reg::Reg::new());
        regs.insert(String::from("ds"), reg::Reg::new());
        regs.insert(String::from("ss"), reg::Reg::new());
        regs.insert(String::from("cs"), reg::Reg::new());
        regs.insert(String::from("ip"), reg::Reg::new());

        // Define opcodes
        let mut opcodes: HashMap<u8, Opcode> = HashMap::new();
        // Move opcodes
        opcodes.insert(0x88, Opcode::new(Rc::new(Self::mov_reg), NumArgs::Two, 1, None, false));
        // opcodes.insert(0xA0, Opcode::new(Rc::new(Self::mov_ax), One, 1, None, false));
        opcodes.insert(0xB8, Opcode::new(Rc::new(Self::mov_reg), NumArgs::Two, 1, Some((Placeholder::Reg(0), Some(Placeholder::Ptr))), false));
        for x in 0..7 {
            opcodes.insert(0xB0, Opcode::new(Rc::new(Self::mov_reg), NumArgs::Two, 1, Some((Placeholder::Reg8(x), None)), false));
            opcodes.insert(0xB8, Opcode::new(Rc::new(Self::mov_reg), NumArgs::Two, 1, Some((Placeholder::Reg16(x), None)), false));
        }
        opcodes.insert(0xC8, Opcode::new(Rc::new(Self::mov_imm), NumArgs::Two, 1, None, true));

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
        } else {
            let code = self.read_ip();
            let d = code & 0x01 >> 0;
            let s = code & 0x02 >> 1;
            let opcode = match self.opcodes.get(&code) {
                Some(opcode) => opcode,
                None => self.opcodes.get(&(code & 0xFC)).unwrap()
            };
            self.next_cycles += opcode.cycles;
            let immediate = opcode.immediate;
            let num_args = opcode.num_args;
            let shorthand = opcode.shorthand.clone();

            if let Some((arg1, arg2)) = opcode.shorthand.clone() {
                let arg1_translated = Some(self.translate_placeholder(arg1, s));
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
                        let arg1 = Some(Self::reg_to_arg(reg, s));
                        let arg2 = {
                            if let None = self.src {
                                if immediate {
                                    Some(
                                        if s == 1 {
                                            Arg::Imm16((self.read_ip() as u16) & ((self.read_ip() as u16) << 8))
                                        } else {
                                            Arg::Imm8(self.read_ip())
                                        }
                                    )
                                } else {
                                    self.translate_mod_rm(mod_bits, rm, s)
                                }
                            } else {
                                None
                            }
                        };

                        if d == 1 && !immediate {
                            if let Some(_) = self.src {
                                self.src = arg1;
                            }
                            if let Some(_) = self.dst {
                                self.dst = arg2;
                            }
                        } else {
                            if let Some(_) = self.src {
                                self.src = arg2;
                            }
                            if let Some(_) = self.dst {
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
        let val = self.ram[self.regs.get("ip").unwrap().value as usize];
        self.regs.get_mut("ip").unwrap().value += 1;
        self.next_cycles += 1;
        val
    }

    fn translate_mod_rm(&mut self, mod_bits: u8, rm: u8, s: u8) -> Option<Arg> {
        match mod_bits {
            0 => Some(Arg::Ptr(self.regs.get(Self::translate_reg16(rm).unwrap().as_str()).unwrap().value)),
            1 => Some(Arg::Ptr(self.regs.get(Self::translate_reg16(rm).unwrap().as_str()).unwrap().value + (self.read_ip() as u16))),
            2 => Some(Arg::Ptr(self.regs.get(Self::translate_reg16(rm).unwrap().as_str()).unwrap().value + (self.read_ip() as u16) + (self.read_ip() as u16))),
            3 => Some(Self::reg_to_arg(rm, s)),
            _ => None
        }
    }

    fn translate_placeholder(&mut self, placeholder: Placeholder, s: u8) -> Arg {
        match placeholder {
            Placeholder::Reg(reg) => {
                if s == 1 {
                    Arg::Reg16(reg)
                } else {
                    Arg::Reg8(reg)
                }
            }
            Placeholder::Reg8(reg) => Arg::Reg8(reg),
            Placeholder::Reg16(reg) => Arg::Reg16(reg),
            Placeholder::Ptr => Arg::Ptr((self.read_ip() as u16) | ((self.read_ip() as u16) << 8))
        }
    }

    fn get_reg_high(&self, num: u8) -> u8 {
        let reg = self.regs.get(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.get_high()
    }

    fn get_reg_low(&self, num: u8) -> u8 {
        let reg = self.regs.get(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.get_low()
    }

    fn set_reg_high(&mut self, num: u8, val: u8) {
        let reg = self.regs.get_mut(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.set_high(val);
    }

    fn set_reg_low(&mut self, num: u8, val: u8) {
        let reg = self.regs.get_mut(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.set_low(val);
    }

    fn reg_to_arg(reg: u8, s: u8) -> Arg {
        if s == 1 {
            Arg::Reg8(reg)
        } else {
            Arg::Reg8(reg)
        }
    }

    fn read_reg(&self, reg: String) -> Option<u16> {
        match self.regs.get(&reg) {
            Some(val) => Some(val.value),
            None => None
        }
    }

    fn read_mem(&self, loc: usize) -> u8 {
        self.ram[loc]
    }

    fn load(&mut self, data: Vec<u8>, loc: usize) {
        for i in 0..data.len() {
            self.ram[loc + i] = data[i];
        }
        self.regs.get_mut("ip").unwrap().value = (loc as u16);
    }

    fn execute_next(&mut self) {
        while { match self.instruction { Some(_) => true, None => false } } {
            self.step();
        }
    }

    fn translate_reg16(num: u8) -> Option<String> {
        match num {
            0 => Some(String::from("ax")),
            1 => Some(String::from("cx")),
            2 => Some(String::from("dx")),
            3 => Some(String::from("bx")),
            4 => Some(String::from("sp")),
            5 => Some(String::from("bp")),
            6 => Some(String::from("si")),
            7 => Some(String::from("di")),
            _ => None
        }
    }

    fn translate_reg8(num: u8) -> Option<String> {
        match num {
            0 => Some(String::from("al")),
            1 => Some(String::from("cl")),
            2 => Some(String::from("dl")),
            3 => Some(String::from("bl")),
            4 => Some(String::from("ah")),
            5 => Some(String::from("ch")),
            6 => Some(String::from("dh")),
            7 => Some(String::from("bh")),
            _ => None
        }
    }
}
