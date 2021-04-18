use crate::cpu::{CPU, Regs};
use crate::cpu::instruction::Instruction;
use crate::cpu::instruction::args::{SrcArg, DstArg};
use crate::cpu::instruction::actions::flags::{advance_di, advance_si};

pub fn in_action(comp: &mut CPU, instruction: Instruction) -> usize {
    let src = instruction.src.unwrap().to_src_arg(comp).unwrap();
    let size = instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap().get_size();
    let address = match src {
        SrcArg::Byte(address) => address as u16,
        SrcArg::Word(address) => address,
        _ => panic!("in can only get a byte port address")
    };

    let res = comp.read_io_mem(address, size);

    comp.write_to_arg(instruction.dst.unwrap(), res).unwrap();

    0
}

pub fn ins(comp: &mut CPU, instruction: Instruction) -> usize {
    comp.instruction.as_mut().map(|s| s.segment = Regs::ES);

    let size = instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap().get_size();

    comp.sub_command(0xEC, instruction.src, Some(DstArg::RegPtr(Regs::DI, size)), 0);

    advance_di(comp, size);

    0
}

pub fn out(comp: &mut CPU, instruction: Instruction) -> usize {
    let dst = instruction.dst.unwrap().to_src_arg(comp).unwrap();
    let val = instruction.src.unwrap().to_src_arg(comp).unwrap();
    let address = match dst {
        SrcArg::Byte(address) => address as u16,
        SrcArg::Word(address) => address,
        _ => panic!("out can only get a byte port address")
    };

    comp.write_io_mem(address, val);

    0
}

pub fn outs(comp: &mut CPU, instruction: Instruction) -> usize {
    comp.instruction.as_mut().map(|s| s.segment = Regs::DS);

    let size = instruction.src.as_ref().unwrap().to_src_arg(comp).unwrap().get_size();

    comp.sub_command(0xEE, Some(DstArg::RegPtr(Regs::SI, size)), instruction.dst, 0);

    advance_si(comp, size);

    0
}
