use crate::cpu::{CPU, Regs, SrcArg, DstArg, CPUFlags};

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

pub fn cdw(comp: &mut CPU) -> usize {
    let ax = comp.regs.get(&Regs::AX).unwrap().value;
    comp.regs.get_mut(&Regs::DX).unwrap().value = if ax & 0x80 == 1 { 0xFF } else { 0x00 };
    0
}

pub fn cdw_mnemonic(_: u8) -> Option<String> {
    Some(String::from("CDW"))
}

pub fn ldw(comp: &mut CPU) -> usize {
    let value = match comp.src {
        Some(SrcArg::DWord(val)) => val,
        _ => panic!("LDS/LES must get a dword as src")
    };
    let seg = comp.seg;
    comp.regs.get_mut(&seg).unwrap().value = (value >> 16) as u16;
    let dst = match comp.dst {
        Some(DstArg::Reg16(reg)) => DstArg::Reg16(reg),
        _ => panic!("LDS/LES must get a Reg16 as dst")
    };
    comp.write_to_arg(dst, SrcArg::Word((value & 0xFFFF) as u16)).unwrap();
    0
}

pub fn lds_mnemonic(_: u8) -> Option<String> {
    Some(String::from("LDS"))
}

pub fn les_mnemonic(_: u8) -> Option<String> {
    Some(String::from("LES"))
}

pub fn lea(comp: &mut CPU) -> usize {
    let new_dst = SrcArg::Word(comp.src_ptr.unwrap());
    let old_dst = comp.dst.unwrap();
    comp.write_to_arg(old_dst, new_dst).unwrap();
    0
}

pub fn lea_mnemonic(_: u8) -> Option<String> {
    Some(String::from("LEA"))
}

pub fn lods(comp: &mut CPU) -> usize {
    let src_loc = comp.regs.get(&Regs::SI).unwrap().value;
    let comp_dst = comp.dst.unwrap();
    match comp.get_src_arg_mut(comp_dst) {
        Some(SrcArg::Word(_)) => {
            let src = comp.get_src_arg_mut(DstArg::Ptr16(src_loc));
            comp.write_to_arg(DstArg::Reg8(0), src.unwrap()).unwrap();
        }
        Some(SrcArg::Byte(_)) => {
            let src = comp.get_src_arg_mut(DstArg::Ptr8(src_loc));
            comp.write_to_arg(DstArg::Reg(Regs::AX), src.unwrap()).unwrap();
        }
        _ => panic!("LODS can only get a byte or word")
    }
    if comp.check_flag(CPUFlags::DIRECTION) {
        comp.regs.get_mut(&Regs::SI).unwrap().value += 1;
    } else  {
        comp.regs.get_mut(&Regs::SI).unwrap().value -= 1;
    }
    0
}

pub fn lods_mnemonic(_: u8) -> Option<String> {
    Some(String::from("LODS"))
}

pub fn nop(_: &mut CPU) -> usize {
                             0
                              }

pub fn nop_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("NOP"))
                                                                     }
