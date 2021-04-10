use crate::cpu::{CPU, CPUFlags, Regs};
use crate::cpu::instruction::actions::alu::{sub_with_carry_8_bit, sub_with_carry_16_bit};
use crate::cpu::instruction::args::{SrcArg, DstArg};
use crate::cpu::instruction::Instruction;

pub fn clc(comp: &mut CPU, _: Instruction) -> usize {
    comp.clear_flag(CPUFlags::CARRY);
    0
}

pub fn cld(comp: &mut CPU, _: Instruction) -> usize {
    comp.clear_flag(CPUFlags::DIRECTION);
    0
}

pub fn cli(comp: &mut CPU, _: Instruction) -> usize {
    comp.clear_flag(CPUFlags::INTERRUPT);
    0
}

pub fn cmc(comp: &mut CPU, _: Instruction) -> usize {
    comp.flip_flag(CPUFlags::CARRY);
    0
}

pub fn cmp(comp: &mut CPU, instruction: Instruction) -> usize {
    let src = instruction.src.clone().unwrap().to_src_arg(comp).unwrap();
    comp.check_carry_sub(src);
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    0
}

pub fn cmps(comp: &mut CPU, instruction: Instruction) -> usize {
    let ptr2 = comp.regs.get(&Regs::DI).unwrap().value;
    comp.instruction.as_mut().map(|mut s| { s.segment = Regs::ES });
    let src_as_dst = instruction.src.clone().unwrap();
    let src = match src_as_dst.to_src_arg(comp).unwrap() {
        SrcArg::Byte(_) => SrcArg::Byte(comp.read_mem_byte_mut(ptr2).unwrap()),
        SrcArg::Word(_) => SrcArg::Word(comp.read_mem_word_mut(ptr2).unwrap()),
        SrcArg::DWord(_) => SrcArg::DWord(comp.read_mem_dword_mut(ptr2).unwrap())
    };

    comp.instruction.as_mut().map(|mut s| { s.segment = Regs::DS; s.src = Some(src_as_dst) });
    comp.check_carry_sub(src);
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    if comp.check_flag(CPUFlags::DIRECTION) {
        comp.regs.get_mut(&Regs::DI).unwrap().value += 1;
        comp.regs.get_mut(&Regs::SI).unwrap().value += 1;
    } else {
        comp.regs.get_mut(&Regs::DI).unwrap().value -= 1;
        comp.regs.get_mut(&Regs::SI).unwrap().value -= 1;
    };
    0
}

pub fn lahf(comp: &mut CPU, _: Instruction) -> usize {
    let new_ah = comp.regs.get(&Regs::FLAGS).unwrap().get_low();
    comp.regs.get_mut(&Regs::AX).unwrap().set_low(new_ah);
    0
}

pub fn rep(comp: &mut CPU, instruction: Instruction) -> usize {
    let cmp;
    let op = match instruction.dst.as_ref().unwrap() {
        DstArg::Opcode(opcode) => match opcode {
            0x6C | 0x7D | 0xA4 | 0xA5 | 0x6E | 0x6F | 0xAA | 0xAB => {
                cmp = false;
                *opcode
            },
            0xA6 | 0xA7 | 0xAE | 0xAF => {
                cmp = true;
                *opcode
            }
            _ => panic!("rep only accepts string operation opcodes")
        },
        _ => panic!("rep only accepts string operation opcodes")
    };
    while comp.regs.get(&Regs::CX).unwrap().value != 0 && if cmp { !comp.check_flag(CPUFlags::ZERO) } else { true } {
        comp.sub_command(op, instruction.src.clone(), instruction.dst.clone(), 0);
        comp.regs.get_mut(&Regs::CX).unwrap().value -= 1;
    }
    0
}

pub fn rep_mnemonic(instruction: Instruction) -> String {
    match instruction.dst {
        Some(DstArg::Opcode(op)) => match op {
            0x6C | 0x7D | 0xA4 | 0xA5 | 0x6E | 0x6F | 0xAA | 0xAB => "rep",
            0xA6 | 0xA7 | 0xAE | 0xAF => "repe",
            _ => panic!("rep only accepts string operation opcodes")
        }
        _ => panic!("rep only accepts string operation opcodes")
    }.to_string()
}

pub fn repne(comp: &mut CPU, instruction: Instruction) -> usize {
    let op = match instruction.dst.as_ref().unwrap() {
        DstArg::Opcode(opcode) => match opcode {
            0xA6 | 0xA7 | 0xAE | 0xAF => *opcode,
            _ => panic!("repne only accepts string comparison opcodes")
        },
        _ => panic!("repne only accepts string comparison opcodes")
    };
    while comp.regs.get(&Regs::CX).unwrap().value != 0 || comp.check_flag(CPUFlags::ZERO) {
        comp.sub_command(op, instruction.src.clone(), instruction.dst.clone(), 0);
        comp.regs.get_mut(&Regs::CX).unwrap().value -= 1;
    }
    0
}
