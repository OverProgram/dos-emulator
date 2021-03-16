use crate::cpu::{CPU, CPUFlags};
use crate::cpu::alu::{sub_with_carry_8_bit, sub_with_carry_16_bit};

pub fn clc(comp: &mut CPU) -> usize {
    comp.clear_flag(CPUFlags::CARRY);
    0
}

pub fn clc_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CLC"))
}

pub fn cld(comp: &mut CPU) -> usize {
    comp.clear_flag(CPUFlags::DIRECTION);
    0
}

pub fn cld_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CLD"))
}

pub fn cli(comp: &mut CPU) -> usize {
    comp.clear_flag(CPUFlags::INTERRUPT);
    0
}

pub fn cli_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CLI"))
}

pub fn cmc(comp: &mut CPU) -> usize {
    comp.flip_flag(CPUFlags::CARRY);
    0
}

pub fn cmc_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CMC"))
}

pub fn cmp(comp: &mut CPU) -> usize {
    comp.check_carry_sub(comp.src.clone().unwrap());
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    0
}

pub fn cmp_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CMP"))
}
