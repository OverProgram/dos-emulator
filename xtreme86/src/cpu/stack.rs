use super::{CPU};
use crate::cpu::{Regs, DstArg, SrcArg};


pub fn push(comp: &mut CPU) -> usize {
    let arg = comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap();
    comp.seg = Regs::SS;
    comp.write_to_arg(DstArg::Ptr16(comp.read_reg(Regs::SP).unwrap() - 1), arg).expect("Err");
    comp.regs.get_mut(&Regs::SP).unwrap().value -= 2;
    1
}

pub fn push_mnemonic(_: u8) -> Option<String> {
                                            Some(String::from("PUSH"))
                                                                       }

pub fn pop(comp: &mut CPU) -> usize {
    let val = SrcArg::Word(comp.read_mem_word_seg(comp.read_reg(Regs::SP).unwrap() + 1, Regs::SS).unwrap());
    comp.write_to_arg(comp.dst.clone().unwrap(), val).unwrap();
    comp.regs.get_mut(&Regs::SP).unwrap().value += 2;
    1
}

pub fn pop_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("POP"))
                                                                     }

pub fn call(comp: &mut CPU) -> usize {
    comp.sub_command(0xFF, None, Some(DstArg::Reg(Regs::IP)), 0b110);
    comp.sub_command(0xE9, comp.src.clone(), comp.dst.clone(), 0);
    0
}

pub fn call_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CALL"))
                                                                       }

pub fn ret(comp: &mut CPU) -> usize {
    comp.sub_command(0x8F, None, Some(DstArg::Reg(Regs::IP)), 0b000);
    comp.sub_command(0xE9, None, Some(DstArg::Reg(Regs::IP)), 0b000);
    0
}

pub fn ret_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("RET"))
                                                                     }

