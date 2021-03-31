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

pub fn far_call(comp: &mut CPU) -> usize {
    comp.sub_command(0xFF, None, Some(DstArg::Reg(Regs::CS)), 0b110);
    comp.sub_command(0xFF, None, Some(DstArg::Reg(Regs::IP)), 0b110);
    // let tmp_dst = comp.dst.clone().unwrap();
    // let comp_dst = if let DstArg::Imm16(val) = tmp_dst {
    //     DstArg::Ptr32(val)
    // } else {
    //     tmp_dst
    // };
    // let dst = comp.get_src_arg_mut(comp_dst);
    // if let Some(SrcArg::DWord(destination)) = dst {
    //     let cs = (destination >> 16) as u16;
    //     let ip = (destination & 0xFFFF) as u16;
    //     comp.regs.get_mut(&Regs::CS).unwrap().value = cs;
    //     comp.regs.get_mut(&Regs::IP).unwrap().value = ip;
    // }
    let arg = comp.dst;
    comp.sub_command(0xFF, None, arg, 0b101);
    0
}

pub fn near_call(comp: &mut CPU) -> usize {
    comp.sub_command(0xFF, None, Some(DstArg::Reg(Regs::IP)), 0b110);
    match comp.dst.clone().unwrap() {
        DstArg::Imm16(val) => {
            comp.sub_command(0xE9, None, Some(DstArg::Imm16(val)), 0);
        },
        _ => {
            let dst = comp.dst.clone().unwrap();
            let val_src = comp.get_src_arg_mut(dst.clone());
            if let Some(src) = val_src {
                comp.write_to_arg(DstArg::Reg(Regs::IP), src);
            }
        }
    }
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

pub fn enter(comp: &mut CPU) -> usize {
    let dst_arg = comp.dst.unwrap();
    let dst = match comp.get_src_arg_mut(dst_arg) {
        Some(SrcArg::Word(val)) => val,
        _ => panic!("First operand for ENTER must be a word")
    };
    let level = match comp.src {
        Some(SrcArg::Byte(val)) => val % 13,
        _ => panic!("Second operand for ENTER must be a byte")
    };
    let frame_ptr = comp.regs.get(&Regs::SP).unwrap().value;
    if level > 0 {
        for _ in 1..level {
            let new_bp = comp.regs.get(&Regs::BP).unwrap().value - 2;
            comp.regs.get_mut(&Regs::BP).unwrap().value = new_bp;
            comp.sub_command(0xFE, None, Some(DstArg::Ptr16(new_bp)), 0b110);
        }
        comp.sub_command(0xFE, None, Some(DstArg::Imm16(frame_ptr)), 0b110);
    }
    comp.regs.get_mut(&Regs::BP).unwrap().value = frame_ptr;
    let new_sp = comp.regs.get(&Regs::SP).unwrap().value - dst;
    comp.regs.get_mut(&Regs::SP).unwrap().value = new_sp;
    0
}

pub fn enter_mnemonic(_: u8) -> Option<String> {
    Some(String::from("ENTER"))
}
