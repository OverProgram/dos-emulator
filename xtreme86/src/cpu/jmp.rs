use super::{CPU};
use crate::cpu::{DstArg, Regs};
use std::rc::Rc;

impl CPU {
    pub fn jmp(&mut self) -> usize {
        let val = match self.dst.clone().unwrap() {
            DstArg::Imm16(val) => val as i16,
            DstArg::Imm8(val) => (val as i8) as i16,
            _ => 0
        };
        if val > 0 {
            self.regs.get_mut(&Regs::IP).unwrap().value += val as u16
        } else {
            self.regs.get_mut(&Regs::IP).unwrap().value -= -val as u16
        }
        0
    }

    pub fn jmp_mnemonic(_: u8) -> Option<String> {
        Some(String::from("JMP"))
    }

    pub fn cond_jmp(condition: Box<dyn Fn(&Self) -> bool>) -> Rc<dyn Fn(&mut Self) -> usize> {
        Rc::new(move |this| {
            if condition(this) {
                this.jmp();
            }
            0
        })
    }

    pub fn cond_jmp_mnemonic(cond_text: String) -> Rc<dyn Fn(u8) -> Option<String>> {
        Rc::new(|_| {
            Some(format!("J{}", cond_text))
        })
    }
}
