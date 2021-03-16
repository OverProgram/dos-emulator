use crate::cpu::{CPU, CPUFlags};

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
