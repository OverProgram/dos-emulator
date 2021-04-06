use enumflags2::{BitFlags, bitflags};
use crate::cpu::{CPU, Regs};
use std::option::Option::Some;
use std::rc::Rc;
use crate::cpu::instruction::Instruction;

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpcodeFlags {
    Immediate = 0x0001,
    SizeMismatch = 0x0002,
    Nop = 0x0004,
    ForceWord = 0x0008,
    ForceByte = 0x0010,
    ForceDWord = 0x0020,
    ForceDirection = 0x0040,
}

#[derive(Clone, Copy, Debug)]
pub enum NumArgs {
    Zero,
    One,
    Two
}

#[derive(Clone, Copy, Debug)]
pub enum Placeholder {
    Reg8(u8),
    Reg16(u8),
    Reg(u8),
    Byte(u8),
    Word(u16),
    Imm,
    Ptr
}

pub type MnemonicFunc = Rc<dyn Fn(u8) -> String>;

#[derive(Clone)]
pub enum Mnemonic {
    Static(String),
    Dynamic(MnemonicFunc)
}

impl Mnemonic {
    pub fn get(self, reg_bits: u8) -> String {
        match self {
            Mnemonic::Static(val) => val,
            Mnemonic::Dynamic(func) => func(reg_bits)
        }
    }
}

pub type OpcodeAction = Rc<dyn Fn(&mut CPU, Instruction) -> usize>;

#[derive(Clone)]
pub struct Opcode {
    pub num_args: NumArgs,
    pub shorthand1: Option<Placeholder>,
    pub shorthand2: Option<Placeholder>,
    pub flags: BitFlags<OpcodeFlags>,
    pub segment: Regs,
    pub action: OpcodeAction,
    pub mnemonic: Mnemonic
}

impl Opcode {
    // fn new(num_args: NumArgs, action: OpcodeAction) -> Self {
    //     Self {
    //         num_args,
    //         action,
    //         flags: BitFlags::EMPTY,
    //         shorthand1: None,
    //         shorthand2: None,
    //         segment: Regs::DS,
    //         mnemonic: Mnemonic::Static(String::from(""))
    //     }
    // }

    // fn set_flags(mut self, flags: BitFlags<OpcodeFlags>) -> Self {
    //     self.flags = flags;
    //     self
    // }
    //
    // fn set_seg(mut self, segment: Regs) -> Self {
    //     self.segment = segment;
    //     self
    // }
    //
    // fn set_placeholders(mut self, shorthand1: Option<Placeholder>, shorthand2: Option<Placeholder>) -> Self {
    //     self.shorthand1 = shorthand1;
    //     self.shorthand2 = shorthand2;
    //     self
    // }
    //
    // fn set_mnemonic_func(mut self, mnemonic: MnemonicFunc) -> Self {
    //     self.mnemonic = Mnemonic::Dynamic(mnemonic);
    //     self
    // }
    //
    // fn set_mnemonic_str(mut self, mnemonic: &str) -> Self {
    //     self.mnemonic = Mnemonic::Static(String::from(mnemonic));
    //     self
    // }

    pub fn has_shorthand(&self) -> bool {
        if let Some(_) = self.shorthand1 {
            true
        } else if let Some(_) = self.shorthand2 {
            true
        } else {
            false
        }
    }
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Opcode")
            .field("num_args", &self.num_args)
            .field("flags", &self.flags)
            .field("segment", &self.segment)
            .finish()
    }
}