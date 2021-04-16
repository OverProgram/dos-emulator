use crate::cpu::CPU;
use crate::cpu::instruction::Instruction;
use crate::cpu::instruction::args::SrcArg;

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

pub fn out(comp: &mut CPU, instruction: Instruction) -> usize {
    0
}
