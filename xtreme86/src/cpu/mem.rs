use super::{CPU};
use crate::cpu::Regs;

pub fn mov(comp: &mut CPU) -> usize {
    comp.write_to_arg(comp.dst.clone().unwrap(), comp.src.clone().unwrap()).unwrap();
    0
}

pub fn mov_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("MOV"))
                                                                     }

pub fn cbw(comp: &mut CPU) -> usize {
    let al = comp.regs.get(&Regs::AX).unwrap().get_low();
    comp.regs.get_mut(&Regs::AX).unwrap().value = al as u16;
    0
}

pub fn cbw_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CBW"))
}

pub fn nop(_: &mut CPU) -> usize {
                             0
                              }

pub fn nop_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("NOP"))
                                                                     }
