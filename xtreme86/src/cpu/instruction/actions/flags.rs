use crate::cpu::{CPU, CPUFlags, Regs};
use crate::cpu::instruction::actions::alu::{sub_with_carry_8_bit, sub_with_carry_16_bit};
use crate::cpu::instruction::args::{SrcArg, DstArg, Size};
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
    comp.set_flag(CPUFlags::INTERRUPT);
    0
}

pub fn stc(comp: &mut CPU, _: Instruction) -> usize {
    comp.set_flag(CPUFlags::CARRY);
    0
}

pub fn std(comp: &mut CPU, _: Instruction) -> usize {
    comp.set_flag(CPUFlags::DIRECTION);
    0
}

pub fn sti(comp: &mut CPU, _: Instruction) -> usize {
    comp.set_flag(CPUFlags::INTERRUPT);
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

pub fn test(comp: &mut CPU, _: Instruction) -> usize {
    let res = comp.operation_2_args(|src, dst| src & dst, |src, dst| src & dst);
    comp.check_flags_in_result(&res, CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::PARITY);
    comp.clear_flag(CPUFlags::OVERFLOW | CPUFlags::CARRY);

    0
}

pub fn advance_di(comp: &mut CPU, size: Size) {
    let advance = match size {
        Size::Byte => 1,
        Size::Word => 2,
        Size::DWord => panic!("string operations only support byte and word")
    };


    let new_di = if comp.check_flag(CPUFlags::DIRECTION) {
        comp.regs.get(&Regs::DI).unwrap().value.wrapping_sub(advance)
    } else {
        comp.regs.get(&Regs::DI).unwrap().value.wrapping_add(advance)
    };

    comp.regs.get_mut(&Regs::DI).unwrap().value = new_di;
}

pub fn advance_si(comp: &mut CPU, size: Size) {
    let advance = match size {
        Size::Byte => 1,
        Size::Word => 2,
        Size::DWord => panic!("string operations only support byte and word")
    };


    let new_si = if comp.check_flag(CPUFlags::DIRECTION) {
        comp.regs.get(&Regs::SI).unwrap().value.wrapping_sub(advance)
    } else {
        comp.regs.get(&Regs::SI).unwrap().value.wrapping_add(advance)
    };

    comp.regs.get_mut(&Regs::SI).unwrap().value = new_si;
}

pub fn cmps(comp: &mut CPU, instruction: Instruction) -> usize {
    let ptr2 = comp.regs.get(&Regs::DI).unwrap().value;
    comp.instruction.as_mut().map(|mut s| { s.segment = Regs::ES });
    let src_as_dst = instruction.dst.clone().unwrap();
    let src_dst = match src_as_dst.to_src_arg(comp).unwrap() {
        SrcArg::Byte(_) => DstArg::Imm8(comp.read_mem_byte_mut(ptr2).unwrap()),
        SrcArg::Word(_) => DstArg::Imm16(comp.read_mem_word_mut(ptr2).unwrap()),
        _ => panic!("cmps can only accept byte or word")
    };
    let src = src_dst.to_src_arg(comp).unwrap();

    comp.instruction.as_mut().map(|mut s| { s.segment = Regs::DS; s.dst = Some(DstArg::RegPtr(Regs::SI, src.clone().get_size())); s.src = Some(src_dst) });
    comp.check_carry_sub(src);
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);

    let size = src.get_size();
    advance_di(comp, size);
    advance_si(comp, size);

    0
}

pub fn scas(comp: &mut CPU, instruction: Instruction) -> usize {
    let size = instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap().get_size();
    let src_dst = DstArg::RegPtr(Regs::DI, size);
    let src = src_dst.to_src_arg(comp).unwrap();
    comp.check_carry_sub(src);
    comp.instruction.as_mut().map(move |s| s.src = Some(src_dst));
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst,src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);

    advance_di(comp, size);

    0
}

pub fn lahf(comp: &mut CPU, _: Instruction) -> usize {
    let new_ah = comp.regs.get(&Regs::FLAGS).unwrap().get_low();
    comp.regs.get_mut(&Regs::AX).unwrap().set_high(new_ah);
    0
}

pub fn sahf(comp: &mut CPU, _: Instruction) -> usize {
    let new_flags= comp.regs.get(&Regs::AX).unwrap().get_high();
    comp.regs.get_mut(&Regs::FLAGS).unwrap().set_low(new_flags);
    0
}

//TODO: Test
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

//TODO: Test
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
