mod reg;
mod mem;

use std::collections::HashMap;
use crate::cpu::Instruction::{MOVImm, MOVReg, MOVAx};
use crate::cpu::NumArgs::{Two, One};
use crate::cpu::Arg::{Reg8, Reg16};

enum NumArgs {
    Zero,
    One,
    Two
}

enum Arg {
    Reg8(String),
    Reg16(String),
    Imm8(u8),
    Imm16(u16),
    Ptr(u16)
}

#[derive(PartialEq, Eq, Hash)]
pub enum Instruction {
    MOVImm,
    MOVReg,
    MOVAx
}

pub enum AddressingMode {
    Immediate,
    Direct,
    Indirect,
    IndirectIndex,
    Relative,
    SIB
}

struct Opcode {
    instruction: Instruction,
    num_args: NumArgs,
    cycles: usize,
    shorthand: Option<(Arg, Option<Arg>)>
}

impl Opcode {
    fn new(instruction: Instruction, num_args: NumArgs, cycles: usize, shorthand: Option<(Arg, Option<Arg>)>) -> Self {
        Self {
            instruction,
            num_args,
            cycles,
            shorthand
        }
    }
}

struct CPU {
    ram: Vec<u8>,
    regs: HashMap<String, reg::Reg>,
    opcodes: HashMap<u8, Opcode>,
    operations: HashMap<Instruction, Box<dyn Fn(&mut CPU) -> usize>>,
    cycles_until_op: usize,
    arg1: Option<Arg>,
    arg2: Option<Arg>
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
        opcodes.insert(0x88, Opcode::new(MOVReg, Two, 1, None));
        opcodes.insert(0xA0, Opcode::new(MOVAx, One, 1, None));
        for x in 0..7 {
            opcodes.insert(0xB0, Opcode::new(MOVReg, Two, 1, Some((Reg8(Self::translate_reg16(x).unwrap()), None))));
            opcodes.insert(0xB8, Opcode::new(MOVReg, Two, 1, Some((Reg16(Self::translate_reg16(x).unwrap()), None))));
        }
        opcodes.insert(0xC8, Opcode::new(MOVImm, Two, 1, None));

        // Bind Instruction enum values to functions
        let mut operations: HashMap<Instruction, Box<dyn Fn(&mut CPU) -> usize>> = HashMap::new();
        operations.insert(MOVReg, Box::new(Self::mov_reg));
        operations.insert(MOVImm, Box::new(Self::mov_imm));
        operations.insert(MOVAx, Box::new(Self::mov_ax));

        Self {
            ram,
            regs,
            opcodes,
            operations,
            cycles_until_op: 0,
            arg1: None,
            arg2: None
        }
    }

    pub fn step(&self) {}

    fn get_reg_high(&self, num: u8) -> u8 {
        let reg = self.regs.get(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.get_high()
    }

    fn get_reg_low(&self, num: u8) -> u8 {
        let reg = self.regs.get(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.get_low()
    }

    fn set_reg_high(&mut self, num: u8, val: u8) {
        let mut reg = self.regs.get(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.set_high(val);
    }

    fn set_reg_low(&mut self, num: u8, val: u8) {
        let mut reg = self.regs.get(Self::translate_reg16(num % 4).unwrap().as_str()).unwrap();
        reg.set_low(val);
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
}
