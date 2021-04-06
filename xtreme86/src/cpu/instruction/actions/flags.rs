use crate::cpu::{CPU, CPUFlags, Regs};
use crate::cpu::instruction::actions::alu::{sub_with_carry_8_bit, sub_with_carry_16_bit};
use crate::cpu::instruction::args::SrcArg;
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
