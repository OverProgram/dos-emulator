use super::{CPU};
use crate::cpu::{DstArg, Regs};
use std::rc::Rc;


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

