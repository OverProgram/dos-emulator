use super::{CPU};
use crate::cpu::{Regs, DstArg, SrcArg};

impl CPU {
    pub fn push(&mut self) -> usize {
        let arg = self.get_src_arg(self.dst.clone().unwrap()).unwrap();
        self.seg = Regs::SS;
        self.write_to_arg(DstArg::Ptr16(self.read_reg(Regs::SP).unwrap() - 1), arg).expect("Err");
        self.regs.get_mut(&Regs::SP).unwrap().value -= 2;
        1
    }

    pub fn pop(&mut self) -> usize {
        let val = SrcArg::Word(self.read_mem_word_seg(self.read_reg(Regs::SP).unwrap() + 1, Regs::SS).unwrap());
        self.write_to_arg(self.dst.clone().unwrap(), val);
        self.regs.get_mut(&Regs::SP).unwrap().value += 2;
        1
    }
}
