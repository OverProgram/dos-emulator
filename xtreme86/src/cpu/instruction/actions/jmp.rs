use crate::cpu::{CPU, Regs};
use std::rc::Rc;
use crate::cpu::instruction::args::{DstArg, SrcArg, Size};
use crate::cpu::instruction::Instruction;


pub fn jmp(comp: &mut CPU, instruction: Instruction) -> usize {
    let val = match instruction.dst.clone().unwrap() {
        DstArg::Imm16(val) => val as i16,
        DstArg::Imm8(val) => (val as i8) as i16,
        _ => 0
    };
    if val > 0 {
        comp.regs.get_mut(&Regs::IP).unwrap().value += val as u16
    } else {
        comp.regs.get_mut(&Regs::IP).unwrap().value -= -val as u16
    }
    0
}

//TODO: Test
pub fn jmp_far(comp: &mut CPU, instruction: Instruction) -> usize {
    let tmp_dst = instruction.dst.unwrap();
    let comp_dst = if let DstArg::Imm16(val) = tmp_dst {
        DstArg::Ptr(val, Size::Word)
    } else {
        tmp_dst
    };
    let dst = comp_dst.to_src_arg(comp);
    if let Some(SrcArg::DWord(destination)) = dst {
        let cs = (destination >> 16) as u16;
        let ip = (destination & 0xFFFF) as u16;
        comp.regs.get_mut(&Regs::CS).unwrap().value = cs;
        comp.regs.get_mut(&Regs::IP).unwrap().value = ip;
    }
    0
}

pub fn cond_jmp(condition: Box<dyn Fn(&CPU) -> bool>) -> Rc<dyn Fn(&mut CPU, Instruction) -> usize> {
    Rc::new(move |this, instruction| {
        if condition(this) {
            this.sub_command(0xE9, instruction.src.clone(), instruction.dst.clone(), 0);
        }
        0
    })
}

pub fn lop(condition: Box<dyn Fn(&CPU) -> bool>) -> Rc<dyn Fn(&mut CPU, Instruction) -> usize> {
    Rc::new(move |this, instruction| {
        let new_cx = this.regs.get(&Regs::CX).unwrap().value.wrapping_sub(1);
        this.regs.get_mut(&Regs::CX).unwrap().value = new_cx;
        if this.regs.get(&Regs::CX).unwrap().value != 0 && condition(this) {
            this.sub_command(0xE9, None, instruction.dst.clone(), 0);
        }
        0
    })
}
