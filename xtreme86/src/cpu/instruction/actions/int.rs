use crate::cpu::{CPU, Regs, exceptions};
use crate::cpu::instruction::args::{SrcArg, DstArg};
use crate::cpu::instruction::Instruction;

pub fn int_req(comp: &mut CPU, instruction: Instruction) -> usize {
    let num = get_int_num(comp, instruction);
    comp.irq = Some(num);
    0
}

pub fn int_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("INT"))
                                                                     }

fn get_int_num(comp: &mut CPU, instruction: Instruction) -> u8 {
    match instruction.dst.clone().unwrap().to_src_arg(comp).unwrap() {
        SrcArg::Byte(val) => Some(val),
        _ => None
    }.unwrap()
}

pub fn int(comp: &mut CPU) -> usize {
    let tmp_es = comp.read_reg(Regs::ES).unwrap();

    comp.sub_command(0xFF, None, Some(DstArg::Reg(Regs::FLAGS)), 0b110);
    comp.sub_command(0xFF, None, Some(DstArg::Reg(Regs::CS)), 0b110);
    comp.sub_command(0xFF, None, Some(DstArg::Reg(Regs::IP)), 0b110);

    let num = comp.irq.unwrap();
    comp.irq = None;

    let new_cs = comp.read_mem_word_seg((num as u16) * 4 + 2, Regs::ES).unwrap();
    let new_ip = comp.read_mem_word_seg((num as u16) * 4, Regs::ES).unwrap();
    comp.write_to_arg(DstArg::Reg(Regs::CS), SrcArg::Word(new_cs)).unwrap();
    comp.write_to_arg(DstArg::Reg(Regs::IP), SrcArg::Word(new_ip)).unwrap();

    comp.set_reg(Regs::ES, tmp_es);
    0
}

pub fn iret(comp: &mut CPU, instruction: Instruction) -> usize {
    comp.sub_command(0x8F, None, Some(DstArg::Reg(Regs::IP)), 0b110);
    comp.sub_command(0x8F, None, Some(DstArg::Reg(Regs::CS)), 0b110);
    comp.sub_command(0x8F, None, Some(DstArg::Reg(Regs::FLAGS)), 0b110);
    0
}

pub fn iret_mnemonic(_: u8) -> Option<String> {
                                            Some(String::from("IRET"))
                                                                       }

pub fn bound(comp: &mut CPU, instruction: Instruction) -> usize {
    if let Some(SrcArg::DWord(bounds)) = instruction.src.clone().unwrap().to_src_arg(comp) {
        match instruction.dst.clone().unwrap() {
            DstArg::Reg16(_) | DstArg::Reg(_) => (),
            _ => comp.except(exceptions::INVALID_OPCODE).unwrap()
        }
        let lower_bound = (bounds & 0xFFFF) as u16;
        let upper_bound = (bounds >> 16) as u16;
        let arg = instruction.dst.clone().unwrap().to_src_arg(comp);
        if let Some(SrcArg::Word(val)) = arg {
            if val > upper_bound || val < lower_bound {
                comp.except(exceptions::BOUND).unwrap();
            }
        }
    }
    0
}

pub fn bound_mnemonic(_: u8) -> Option<String> {
    Some(String::from("BOUND"))
}
