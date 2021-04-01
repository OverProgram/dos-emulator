use enumflags2::{BitFlags, bitflags, make_bitflags};
use crate::cpu::{CPU, Regs};
use std::option::Option::Some;

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

type OpcodeAction = Box<dyn Fn(&mut CPU) -> usize>;
type OpcodeMnemonic = Box<dyn Fn(u8) -> String>;

#[derive(Copy, Clone)]
pub struct Opcode {
    pub num_args: NumArgs,
    pub shorthand1: Option<Placeholder>,
    pub shorthand2: Option<Placeholder>,
    pub flags: BitFlags<OpcodeFlags>,
    pub segment: Regs,
}

impl Opcode {
    const fn new(num_args: NumArgs) -> Self {
        Self {
            num_args,
            flags: BitFlags::EMPTY,
            shorthand1: None,
            shorthand2: None,
            segment: Regs::DS
        }
    }

    const fn set_flags(mut self, flags: BitFlags<OpcodeFlags>) -> Self {
        self.flags = flags;
        self
    }

    const fn set_seg(mut self, segment: Regs) -> Self {
        self.segment = segment;
        self
    }

    const fn set_placeholders(mut self, shorthand1: Option<Placeholder>, shorthand2: Option<Placeholder>) -> Self {
        self.shorthand1 = shorthand1;
        self.shorthand2 = shorthand2;
        self
    }

    fn has_shorthand(&self) -> bool {
        if let Some(_) = self.shorthand1 {
            true
        } else if let Some(_) = self.shorthand2 {
            true
        } else {
            false
        }
    }
}

pub fn get_opcode_data() -> [Opcode; u8::MAX as usize] {
    let mut array = [Opcode::new(NumArgs::Zero); u8::MAX as usize];
    let mut i: usize = 0;
    // NOP
    array[0x90] = Opcode::new(NumArgs::Zero)
        .set_flags(make_bitflags!(OpcodeFlags::{ Nop }));
    // MOV
    array[0x88] = Opcode::new(NumArgs::Two);
    array[0xA0] = Opcode::new(NumArgs::Two)
        .set_placeholders(Some(Placeholder::Reg(0)), Some(Placeholder::Ptr));
    i = 0;
    while i < 8 {
        array[0xB0 + i] = Opcode::new(NumArgs::Two)
            .set_placeholders(Some(Placeholder::Reg8(i as u8)), Some(Placeholder::Imm))
            .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
        array[0xB8 + i] = Opcode::new(NumArgs::Two)
            .set_placeholders(Some(Placeholder::Reg16(i as u8)), Some(Placeholder::Imm))
            .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
        i += 1
    }
    array[0xC6] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
    // LDS
    array[0xC5] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ ForceDWord }));
    // LES
    array[0xC4] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ ForceDWord }));
    // LEA
    array[0x8D] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ ForceDirection }));
    // LODSB
    array[0xAC] = Opcode::new(NumArgs::One)
        .set_placeholders(Some(Placeholder::Byte(0)), None);
    // LODSW
    array[0xAD] = Opcode::new(NumArgs::One)
        .set_placeholders(Some(Placeholder::Word(0)), None);
    // ADD, SUB, XOR, AND, OR
    let offsets = [0x00, 0x28, 0x30, 0x20, 0x08];
    i = 0;
    while i < offsets.len() {
        array[0x00 + offsets[i]] = Opcode::new(NumArgs::Two);
        array[0x04 + offsets[i]] = Opcode::new(NumArgs::Two)
            .set_placeholders(Some(Placeholder::Reg(0)), Some(Placeholder::Imm))
            .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
    }
    // ADD, OR, ADC, AND, SUB, XOR, CMP
    array[0x80] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
    // INC, DEC
    i = 0;
    while i < 16 {
        array[0x40 + i] = Opcode::new(NumArgs::Zero)
            .set_placeholders(Some(Placeholder::Reg16((i % 8) as u8)), None);
        i += 1;
    }
    // ADD, OR, ADC, AND, SUB, XOR, CMP
    array[0x83] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | SizeMismatch }));
    // INC, DEC, CALL (near), CALL (far), JMP (near), JMP (far), PUSH
    array[0xFE] = Opcode::new(NumArgs::One);
    array[0xF6] = Opcode::new(NumArgs::One);
    // NOT, NEG, MUL IMUL, DIV, IDIV
    array[0xD5] = Opcode::new(NumArgs::One)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | ForceByte }));
    // ADC
    array[0x14] = Opcode::new(NumArgs::Two)
        .set_placeholders(Some(Placeholder::Reg8(0)), Some(Placeholder::Imm))
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
    array[0x10] = Opcode::new(NumArgs::Two);
    // PUSH, POP
    i = 0;
    while i < 16 {
        array[0x50 + i] = Opcode::new(NumArgs::One)
            .set_placeholders(Some(Placeholder::Reg16((i % 8) as u8)), None);
        i += 1;
    }
    array[0x8F] = Opcode::new(NumArgs::One);
    // CALL (near)
    array[0xE8] = Opcode::new(NumArgs::One)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | ForceWord }));
    // CALL (far)
    array[0x9A] = Opcode::new(NumArgs::One)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | ForceWord }));
    // RET
    array[0xC3] = Opcode::new(NumArgs::One)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | ForceWord }));
    // ENTER
    array[0xC8] = Opcode::new(NumArgs::Two)
        .set_placeholders(Some(Placeholder::Imm), Some(Placeholder::Imm))
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | SizeMismatch }));
    // Jump opcodes
    // JMP
    array[0xE9] = Opcode::new(NumArgs::One)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
    // Jcond
    i = 0;
    while i < 16 {
        array[0x70 + i] = Opcode::new(NumArgs::One)
            .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | SizeMismatch }));
        i += 1;
    }
    // LOOPcond
    i = 0;
    while i < 3 {
        array[0xE0 + i] = Opcode::new(NumArgs::One)
            .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
        i += 1;
    }
    // CMP
    //TODO: check CMP correctness
    array[0x38] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ SizeMismatch }));
    array[0x3C] = Opcode::new(NumArgs::Two)
        .set_placeholders(Some(Placeholder::Reg(0)), Some(Placeholder::Imm))
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
    // CMPS
    array[0xA6] = Opcode::new(NumArgs::Zero)
        .set_placeholders(Some(Placeholder::Reg16(6)), Some(Placeholder::Reg16(7)));
    // INT
    array[0xCD] = Opcode::new(NumArgs::One)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate | ForceByte }));
    array[0xCC] = Opcode::new(NumArgs::One)
        .set_placeholders(Some(Placeholder::Byte(3)), None)
        .set_flags(make_bitflags!(OpcodeFlags::{ Immediate }));
    // BOUND
    //TODO: check BOUND correctness
    array[0x62] = Opcode::new(NumArgs::Two)
        .set_flags(make_bitflags!(OpcodeFlags::{ ForceDWord }));
    array
}
