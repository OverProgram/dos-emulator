use super::{CPU, Arg};

impl CPU {
    pub fn mov_imm(&mut self) -> usize {
        let size;
        let val ;

        match self.src.clone().unwrap() {
            Arg::Imm8(value) => {
                val = value as u16;
                size = false;
            }
            Arg::Imm16(value) => {
                val = value;
                size = true;
            }
            _ => {
                return 0;
            }
        }

        self.mov(val, self.dst.clone().unwrap(), size)
    }

    pub fn mov_reg(&mut self) -> usize {
        let mut size = match self.dst.clone().unwrap() {
            Arg::Reg16(_) | Arg::Imm16(_) => true,
            _ => false
        };
        let mut val;
        let mut cycles = 0;

        match self.src.clone().unwrap() {
            Arg::Reg8(reg) => {
                if reg > 4 {
                    val = self.get_reg_high(reg) as u16;
                } else {
                    val = self.get_reg_low(reg) as u16;
                }
                size = false;
            }
            Arg::Reg16(reg) => {
                val = self.regs.get(&Self::translate_reg16(reg).unwrap()).unwrap().value;
                size = true;
            }
            Arg::Ptr(ptr) => {
                val = self.ram[ptr as usize] as u16;
                if size {
                    val |= (self.ram[(ptr + 1) as usize] as u16) << 8;
                    cycles += 1;
                }
                cycles += 1;
            }
            _ => {
                return 0;
            }
        }

        self.mov(val, self.dst.clone().unwrap(), size) + cycles
    }


    fn mov(&mut self, val: u16, dst: Arg, size: bool) -> usize {
        match dst {
            Arg::Ptr(ptr) => {
                self.ram[ptr as usize] = (val & 0xFF) as u8;
                if size {
                    self.ram[(ptr + 1) as usize] =((val & 0xFF00) >> 8) as u8;
                    2
                } else {
                    1
                }
            }
            Arg::Reg16(reg) => {
                self.regs.get_mut(&Self::translate_reg16(reg).unwrap()).unwrap().value = val;
                0
            }
            Arg::Reg8(reg) => {
                if reg > 4 {
                    self.set_reg_high(reg, val as u8);
                } else {
                    self.set_reg_low(reg, val as u8);
                }
                0
            }
            _ => 0
        }
    }
}
