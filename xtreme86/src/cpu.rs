mod reg;
mod instruction;

use std::collections::HashMap;
use std::fmt::{Debug};
use crate::cpu::instruction::actions::{int, alu};
use crate::cpu::instruction::{InstructionDecoder};
use crate::cpu::instruction::args::{SrcArg, DstArg, Size};
use crate::cpu::instruction::opcode::OpcodeFlags;
use crate::peripheral::Peripheral;

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

pub mod exceptions {
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
        Some((Self::translate_reg16(num % 4)?, if num / 4 == 0 { WordPart::Low } else { WordPart::High }))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WordPart {
    Low,
    High
}

pub struct CPU {
    ram: Vec<u8>,
    regs: HashMap<Regs, reg::Reg>,
    opcodes: [Option<instruction::opcode::Opcode>; 256],
    instruction: Option<instruction::Instruction>,
    next_cycles: usize,
    irq: Option<u8>,
    opcode_address: (u16, u16),
    io_devices: Vec<Box<dyn Peripheral>>,
    io_memory_hooks: HashMap<u16, usize>,
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

        Self {
            ram,
            regs,
            opcodes: instruction::opcode::Opcode::get_opcode_data(),
            instruction: None,
            next_cycles: 0,
            irq: None,
            opcode_address: (0, 0),
            io_devices: Vec::new(),
            io_memory_hooks: HashMap::new(),
        }
    }

    pub fn step(&mut self) {
        if self.next_cycles > 0 {
            self.next_cycles -= 1;
        } else if let Some(opcode) = self.instruction.clone() {
            opcode.exec(self);
            self.instruction = None;
        } else if let Some(_) = self.irq {
            self.next_cycles += int::int(self);
        } else {
            let opcode_address =  (self.regs[&Regs::CS].value, self.regs[&Regs::IP].value);
            self.opcode_address = opcode_address;
            let physical_address = Self::physical_address(opcode_address.0, opcode_address.1) as usize;
            self.instruction.replace(instruction::InstructionDecoder::new(self.opcodes.clone(), &self.ram[physical_address..]).get().unwrap());
            self.next_cycles += self.instruction.clone().unwrap().next_cycles;
            self.regs.get_mut(&Regs::IP).unwrap().value += self.instruction.clone().unwrap().length as u16;
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

    fn read_mem_byte_mut(&mut self, ptr: u16) -> Option<u8> {
        if ptr > self.ram.len() as u16 {
            None
        } else {
            self.next_cycles += 1;
            Some(self.ram[Self::physical_address(self.read_reg(self.instruction.clone().unwrap().segment).unwrap(), ptr) as usize])
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
            let seg_val = self.read_reg(self.instruction.clone().unwrap().segment).unwrap();
            self.ram[Self::physical_address(seg_val, ptr) as usize] = (val & 0xFF) as u8;
            self.next_cycles += 1;
            Ok(())
        }
    }

    fn write_mem_word(&mut self, ptr: u16, val: u16) -> Result<(), &str> {
        self.write_mem_byte(ptr, (val & 0x00FF) as u8).unwrap();
        self.write_mem_byte(ptr + 1, ((val & 0xFF00) >> 8) as u8)
    }


    fn write_mem_dword(&mut self, ptr: u16, val: u32) -> Result<(), &str> {
        self.write_mem_word(ptr, (val &0xFFFF) as u16).unwrap();
        self.write_mem_word(ptr + 2, ((val & 0xFFFF0000) >> 16) as u16)
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

    fn read_io_mem(&mut self, address: u16, size: Size) -> SrcArg {
        let dev_index = self.io_memory_hooks[&address];
        let dev = self.io_devices.get_mut(dev_index).unwrap();
        match size {
            Size::Byte => SrcArg::Byte(dev.handle_mem_read_byte(address)),
            Size::Word => SrcArg::Word(dev.handle_mem_read_word(address)),
            _ => panic!("can only read byte or word from io memory")
        }
    }

    fn write_io_mem(&mut self, address: u16, val: SrcArg) {
        let dev_index = self.io_memory_hooks[&address];
        let dev = self.io_devices.get_mut(dev_index).unwrap();
        match val {
            SrcArg::Byte(byte) => dev.handle_mem_write_byte(address, byte),
            SrcArg::Word(word) => dev.handle_mem_write_word(address, word),
            _ => panic!("can only write byte or word from io memory")
        }
    }

    fn sign_extend(num: u8) -> u16 {
        let sign_bit = (num >> 7) as u16;
        let mut new_num = num as u16;
        for i in 8..16 {
            new_num |= sign_bit << i;
        }
        new_num
    }

    fn write_to_arg(&mut self, arg: DstArg, val_arg: SrcArg) -> Result<(), &str> {
        match arg {
            DstArg::Reg16(reg) => {
                self.regs.get_mut(&Regs::translate_reg16(reg).unwrap()).unwrap().value = match val_arg {
                    SrcArg::Byte(val) => Self::sign_extend(val),
                    SrcArg::Word(val) => val,
                    _ => panic!("invalid operand sizes")
                };
                Ok(())
            },
            DstArg::Reg8(reg_num) => {
                let (reg_enum, part) = Regs::translate_reg8(reg_num).unwrap();
                let reg = self.regs.get_mut(&reg_enum).unwrap();
                let value = if let SrcArg::Byte(val) = val_arg {
                    val
                } else {
                    panic!("invalid operand sizes")
                };
                match part {
                    WordPart::Low => { reg.set_low(value) },
                    WordPart::High => { reg.set_high(value) }
                }
                Ok(())
            },
            DstArg::Reg(reg) => {
                self.regs.get_mut(&reg).unwrap().value = match val_arg {
                    SrcArg::Byte(val) => Self::sign_extend(val),
                    SrcArg::Word(val) => val,
                    _ => panic!("invalid operand sizes")
                };
                Ok(())
            },
            DstArg::Ptr(ptr, size) => {
                size.write_to_mem(self, ptr, val_arg)
            },
            DstArg::RegPtr(reg, size) => {
                let ptr = self.read_reg(reg).unwrap();
                size.write_to_mem(self, ptr, val_arg)
            },
            DstArg::RegPtrImm(reg, imm, size) => {
                let ptr = self.read_reg(reg).unwrap() + imm;
                size.write_to_mem(self, ptr, val_arg)
            },
            DstArg::RegPtrOff(reg1, reg2, size) => {
                let ptr = self.read_reg(reg1).unwrap() + self.read_reg(reg2).unwrap();
                size.write_to_mem(self, ptr, val_arg)
            }
            DstArg::RegPtrOffImm(reg1, reg2, imm, size) => {
                let ptr = self.read_reg(reg1).unwrap() + self.read_reg(reg2).unwrap() + imm;
                size.write_to_mem(self, ptr, val_arg)
            }
            _ => Err("Invalid dst arg")
        }
    }

    fn operation_1_arg<T, U>(&mut self, byte: T, word: U) -> SrcArg where
        T: Fn(u8)-> u8,
        U: Fn(u16) -> u16
    {
        match self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
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
        match self.instruction.clone().unwrap().src.clone().unwrap().to_src_arg(self).unwrap() {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    Some(SrcArg::Word(word(src, dst)))
                } else {
                    None
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    Some(SrcArg::Byte(byte(src, dst)))
                } else if let SrcArg::Word(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    Some(SrcArg::Word(word(src as u16, dst)))
                } else {
                    None
                }
            },
            _ => None
        }.unwrap()
    }

    fn sub_command(&mut self, opcode: u8, src: Option<DstArg>, dst: Option<DstArg>, reg_bits: u8) {
        let instruction = {
            let mut tmp = instruction::Instruction::new();

            tmp.action = Some(InstructionDecoder::get_opcode_from_slice(&self.opcodes, opcode).unwrap().action);
            tmp.src = src;
            tmp.dst = dst;
            tmp.reg_bits = reg_bits;

            tmp
        };

        let tmp_instruction = self.instruction.clone();
        self.instruction.replace(instruction.clone());

        self.next_cycles += instruction.exec(self);

        self.instruction = tmp_instruction;
    }


    fn do_opcode(&mut self, opcode: u8) {
        let instruction = InstructionDecoder::new(self.opcodes.clone(), self.ram.as_slice()).decode(opcode).unwrap();

        let tmp_instruction = self.instruction.clone();
        self.instruction.replace(instruction.clone());

        self.next_cycles += instruction.exec(self);

        self.instruction = tmp_instruction;
    }

    fn check_carry_add(&mut self, arg: SrcArg) {
        match arg {
            SrcArg::Word(src) => {
                if let SrcArg::Word(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    self.check_carry_16_bit(src, dst);
                    self.check_aux_carry_result(src, dst);
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    self.check_carry_8_bit(src, dst);
                } else if let SrcArg::Word(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    self.check_carry_16_bit(src as u16, dst);
                }
            }
            _ => ()
        };
    }

    fn check_flags_in_result(&mut self, result: &SrcArg, flags: u16) {
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

    fn check_sign(&mut self, result: &SrcArg) {
        self.set_flag_if(CPUFlags::SIGN, Self::check_src_arg(result, |val| (val & 0x80) != 0, |val| (val & 0x80) != 0));
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
                if let SrcArg::Word(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    self.check_carry_16_bit(dst, alu::twos_compliment_word(src));
                }
            },
            SrcArg::Byte(src) => {
                if let SrcArg::Byte(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
                    self.check_carry_8_bit(dst, alu::twos_compliment_byte(src));
                } else if let SrcArg::Word(dst) = self.instruction.clone().unwrap().dst.unwrap().to_src_arg(self).unwrap() {
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

    fn check_aux_carry_result(&mut self, arg1: u16, arg2: u16) {
        let result = (arg1 & 0x00FF) + arg2;
        if result & 0xFF00 > 0 {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::AUX_CARRY;
        } else {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !CPUFlags::AUX_CARRY;
        }
    }

    pub fn get_peripheral(&self, dev_index: usize) -> Option<&Box<dyn Peripheral>> {
        self.io_devices.get(dev_index)
    }

    pub fn hook_peripheral(&mut self, dev: Box<dyn Peripheral>) -> usize {
        let index = self.io_devices.len();
        dev.init(self, index);
        self.io_devices.push(dev);
        index
    }

    pub fn hook_io_memory(&mut self, dev_index: usize, address: u16) {
        self.io_memory_hooks.insert(address, dev_index);
    }

    pub fn hook_interrupt(&mut self, dev_index: usize, int_num: u8) {
        self.write_word((int_num as usize) * 4 + 2, 0xFFFF).unwrap();
        self.write_word((int_num as usize) * 4, dev_index as u16).unwrap();
    }

    pub fn read_reg(&self, reg: Regs) -> Option<u16> {
        match self.regs.get(&reg) {
            Some(val) => Some(val.value),
            None => None
        }
    }

    pub fn read_reg_part(&self, reg: Regs, part: WordPart) -> u8 {
        let tmp = &self.regs[&reg];
        match part {
            WordPart::High => tmp.get_high(),
            WordPart::Low => tmp.get_low()
        }
    }

    pub fn set_reg_part(&mut self, reg: Regs, part: WordPart, val: u8) {
        let tmp = self.regs.get_mut(&reg).unwrap();
        match part {
            WordPart::High => tmp.set_high(val),
            WordPart::Low => tmp.set_low(val)
        }
    }

    pub fn probe_mem(&self, loc: usize) -> u8 {
        self.ram[loc]
    }

    pub fn probe_mem_word(&self, loc: usize) -> u16 {
        (self.ram[loc] as u16) | ((self.ram[loc + 1] as u16) << 8)
    }

    pub fn probe_mem_ds(&self, loc: u16) -> u8 {
        self.probe_mem(Self::physical_address(self.regs.get(&Regs::DS).unwrap().value, loc) as usize)
    }

    pub fn probe_mem_es(&self, loc: u16) -> u8 {
        self.probe_mem(Self::physical_address(self.regs.get(&Regs::ES).unwrap().value, loc) as usize)
    }

    pub fn probe_mem_ds_word(&self, loc: u16) -> u16 {
        self.probe_mem_word(Self::physical_address(self.regs.get(&Regs::DS).unwrap().value, loc) as usize)
    }

    pub fn probe_mem_es_word(&self, loc: u16) -> u16 {
        self.probe_mem_word(Self::physical_address(self.regs.get(&Regs::ES).unwrap().value, loc) as usize)
    }

    pub fn write_bytes(&mut self, start_loc: usize, bytes: Vec<u8>) -> Result<(), String> {
        for (i, byte) in bytes.iter().enumerate() {
            *self.ram.get_mut(start_loc + i).map_or(Err("Index out of bounds"), |s| Ok(s))? = *byte;
        }
        Ok(())
    }

    pub fn write_bytes_ds(&mut self, start_loc: u16, bytes: Vec<u8>) -> Result<(), String> {
        let ds = self.regs.get(&Regs::DS).unwrap().value;
        self.write_bytes(Self::physical_address(ds, start_loc) as usize, bytes)
    }

    pub fn write_bytes_es(&mut self, start_loc: u16, bytes: Vec<u8>) -> Result<(), String> {
        let es = self.regs.get(&Regs::ES).unwrap().value;
        self.write_bytes(Self::physical_address(es, start_loc) as usize, bytes)
    }

    pub fn write_word(&mut self, loc: usize, word: u16) -> Result<(), String> {
        self.write_bytes(loc, vec![(word & 0xFF) as u8, (word >> 8) as u8])
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
        while match self.instruction.clone() { Some(instruction) => !instruction.has_flag(OpcodeFlags::Nop), None => true } {
            self.step();
        }
        while self.next_cycles > 0 {
            self.step();
        }
    }

    pub fn run_to_nop_from_ip(&mut self) {
        let ip = self.regs.get(&Regs::IP).unwrap().value;
        self.run_to_nop(ip);
    }

    pub fn set_reg(&mut self, reg: Regs, val: u16) {
        self.regs.get_mut(&reg).unwrap().value = val
    }

    pub fn get_mem_seg(&self, seg: Regs, loc: u16) -> u8 {
        let seg_val = self.read_reg(seg).unwrap();
        self.ram[Self::physical_address(seg_val, loc) as usize]
    }

    pub fn get_instruction_text(&self, loc: usize) -> Option<String> {
        let decoder = instruction::InstructionDecoder::new(self.opcodes.clone(), &self.ram[loc..]);

        Some(decoder.get()?.to_string())
    }

    pub fn physical_address(seg: u16, offset: u16) -> u32 {
        ((seg as u32) << 4) + (offset as u32)
    }

    pub fn address_in_ds(&self, offset: u16) -> u32 {
        Self::physical_address(self.read_reg(Regs::DS).unwrap(), offset)
    }
}
