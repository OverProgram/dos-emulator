use super::{CPU};
use crate::cpu::{DstArg, Regs, SrcArg};
use std::rc::Rc;
use std::net::Shutdown::Read;


pub fn jmp(comp: &mut CPU) -> usize {
    let val = match comp.dst.clone().unwrap() {
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

pub fn jmp_far(comp: &mut CPU) -> usize {
    let tmp_dst = comp.dst.unwrap();
    let comp_dst = if let DstArg::Imm16(val) = tmp_dst {
        DstArg::Ptr16(val)
    } else {
        tmp_dst
    };
    let dst = comp.get_src_arg_mut(comp_dst);
    if let Some(SrcArg::DWord(destination)) = dst {
        let cs = (destination >> 16) as u16;
        let ip = (destination & 0xFFFF) as u16;
        comp.regs.get_mut(&Regs::CS).unwrap().value = cs;
        comp.regs.get_mut(&Regs::IP).unwrap().value = ip;
    }
    0
}

pub fn jmp_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("JMP"))
                                                                     }

pub fn cond_jmp(condition: Box<dyn Fn(&CPU) -> bool>) -> Rc<dyn Fn(&mut CPU) -> usize> {
    Rc::new(move |this| {
        if condition(this) {
            this.sub_command(0xE9, this.src.clone(), this.dst.clone(), 0);
        }
        0
    })
}

pub fn cond_jmp_mnemonic(cond_text: String) -> Rc<dyn Fn(u8) -> Option<String>> {
    Rc::new(move |_| {
        Some(format!("J{}", cond_text))
    })
}

pub fn lop(condition: Box<dyn Fn(&CPU) -> bool>) -> Rc<dyn Fn(&mut CPU) -> usize> {
    Rc::new(move |this| {
        // this.regs.get_mut(&Regs::CX).unwrap().value.wrapping_sub(1);
        let new_cx = this.regs.get(&Regs::CX).unwrap().value.wrapping_sub(1);
        this.regs.get_mut(&Regs::CX).unwrap().value = new_cx;
        // println!("{}", this.regs.get(&Regs::CX).unwrap().value);
        if this.regs.get(&Regs::CX).unwrap().value != 0 && condition(this) {
            println!("jumpin'");
            this.sub_command(0xE9, None, this.dst, 0);
        }
        0
    })
}

pub fn loop_mnemonic(cond_text: String) -> Rc<dyn Fn(u8) -> Option<String>> {
    Rc::new(move |_| {
        Some(format!("LOOP{}", cond_text))
    })
}
