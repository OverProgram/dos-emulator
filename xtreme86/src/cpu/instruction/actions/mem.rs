use crate::cpu::{CPU, Regs, CPUFlags};
use crate::cpu::instruction::args::{SrcArg, DstArg, Size};
use crate::cpu::instruction::Instruction;

pub fn mov(comp: &mut CPU, instruction: Instruction) -> usize {
    let src = instruction.src.clone().unwrap().to_src_arg(comp).unwrap();
    comp.write_to_arg(instruction.dst.clone().unwrap(), src).unwrap();
    0
}

pub fn cbw(comp: &mut CPU, _: Instruction) -> usize {
    let al = comp.regs.get(&Regs::AX).unwrap().get_low();
    comp.regs.get_mut(&Regs::AX).unwrap().value = al as u16;
    0
}

pub fn cdw(comp: &mut CPU, _: Instruction) -> usize {
    let ax = comp.regs.get(&Regs::AX).unwrap().value;
    comp.regs.get_mut(&Regs::DX).unwrap().value = if ax & 0x80 == 1 { 0xFF } else { 0x00 };
    0
}

pub fn ldw(comp: &mut CPU, instruction: Instruction) -> usize {
    let value = match instruction.src.clone().unwrap().to_src_arg(comp) {
        Some(SrcArg::DWord(val)) => val,
        _ => panic!("LDS/LES must get a dword as src")
    };
    let seg = instruction.segment;
    comp.regs.get_mut(&seg).unwrap().value = (value >> 16) as u16;
    let dst = match instruction.dst {
        Some(DstArg::Reg16(reg)) => DstArg::Reg16(reg),
        _ => panic!("LDS/LES must get a Reg16 as dst")
    };
    comp.write_to_arg(dst, SrcArg::Word((value & 0xFFFF) as u16)).unwrap();
    0
}

pub fn lea(comp: &mut CPU, instruction: Instruction) -> usize {
    let new_dst = SrcArg::Word(instruction.src.as_ref().unwrap().to_ptr(comp).unwrap());
    let old_dst = instruction.dst.unwrap();
    comp.write_to_arg(old_dst, new_dst).unwrap();
    0
}

pub fn lods(comp: &mut CPU, instruction: Instruction) -> usize {
    let src_loc = comp.regs.get(&Regs::SI).unwrap().value;
    let comp_dst = instruction.dst.unwrap();
    match comp_dst.to_src_arg(comp) {
        Some(SrcArg::Word(_)) => {
            let src = DstArg::Ptr(src_loc, Size::Word).to_src_arg(comp);
            comp.write_to_arg(DstArg::Reg8(0), src.unwrap()).unwrap();
        }
        Some(SrcArg::Byte(_)) => {
            let src = DstArg::Ptr(src_loc, Size::Byte).to_src_arg(comp);
            comp.write_to_arg(DstArg::Reg(Regs::AX), src.unwrap()).unwrap();
        }
        _ => panic!("LODS can only get a byte or word")
    }
    if comp.check_flag(CPUFlags::DIRECTION) {
        comp.regs.get_mut(&Regs::SI).unwrap().value += 1;
    } else  {
        comp.regs.get_mut(&Regs::SI).unwrap().value -= 1;
    }
    0
}

pub fn nop(_: &mut CPU, _: Instruction) -> usize {
                             0
                              }
