use crate::cpu::{CPU, SrcArg, DstArg, Regs};

impl CPU {
    pub fn int_req(&mut self) -> usize {
        let num = self.get_int_num();
        self.irq = Some(num);
        0
    }

    fn get_int_num(&mut self) -> u8 {
        match self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Byte(val) => Some(val),
            SrcArg::Word(_) => None
        }.unwrap()
    }

    pub fn int(&mut self) -> usize {
        let tmp_es = self.read_reg(Regs::ES).unwrap();

        self.sub_command(0xFF, None, Some(DstArg::Reg(Regs::FLAGS)), 0b110);
        self.sub_command(0xFF, None, Some(DstArg::Reg(Regs::CS)), 0b110);
        self.sub_command(0xFF, None, Some(DstArg::Reg(Regs::IP)), 0b110);

        let num = self.irq.unwrap();
        self.irq = None;

        let new_cs = self.read_mem_word_seg((num as u16) * 4 + 2, Regs::ES).unwrap();
        let new_ip = self.read_mem_word_seg((num as u16) * 4, Regs::ES).unwrap();
        self.write_to_arg(DstArg::Reg(Regs::CS), SrcArg::Word(new_cs));
        self.write_to_arg(DstArg::Reg(Regs::IP), SrcArg::Word(new_ip));

        self.set_reg(Regs::ES, tmp_es);
        0
    }

    pub fn iret(&mut self) -> usize {
        self.sub_command(0x8F, None, Some(DstArg::Reg(Regs::IP)), 0b110);
        self.sub_command(0x8F, None, Some(DstArg::Reg(Regs::CS)), 0b110);
        self.sub_command(0x8F, None, Some(DstArg::Reg(Regs::FLAGS)), 0b110);
        0
    }
}
