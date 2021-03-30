use crate::cpu::{CPU, CPUFlags, Regs, DstArg, SrcArg};
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

pub fn cmps(comp: &mut CPU) -> usize {
    let ptr2 = comp.regs.get(&Regs::DI).unwrap().value;
    comp.seg = Regs::ES;
    let src = match comp.src.clone().unwrap() {
        SrcArg::Byte(_) => {
            SrcArg::Byte(comp.read_mem_byte_mut(ptr2).unwrap())
        }
        SrcArg::Word(_) => {
            SrcArg::Word(comp.read_mem_word_mut(ptr2).unwrap())
        }
        _ => {
            panic!("HOW TF DID YOU EVEN MANAGE TO GET HERE????");
        }
    };
    comp.seg = Regs::DS;
    comp.check_carry_sub(src.clone());
    comp.src = Some(src);
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    if comp.check_flag(CPUFlags::DIRECTION) {
        comp.regs.get_mut(&Regs::DI).unwrap().value += 1;
        comp.regs.get_mut(&Regs::SI).unwrap().value += 1;
    } else {
        comp.regs.get_mut(&Regs::DI).unwrap().value -= 1;
        comp.regs.get_mut(&Regs::SI).unwrap().value -= 1;
    };
    0
}

pub fn cmps_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CMPS"))
}
