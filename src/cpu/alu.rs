use super::{CPU};
use crate::cpu::{SrcArg, DstArg, Regs, CPUFlags};

impl CPU {
    pub fn alu_dispatch_two_args(&mut self) -> usize {
        match self.reg_bits {
            0b000 => self.add(),
            0b001 => self.or(),
            0b100 => self.and(),
            0b101 => self.sub(),
            0b110 => self.xor(),
            _ => 0
        }
    }

    pub fn alu_dispatch_one_arg(&mut self) -> usize {
        match self.reg_bits {
            0b000 => self.inc(),
            0b001 => self.dec(),
            0b110 => self.push(),
            _ => 0
        }
    }

    pub fn mul_dispatch(&mut self) -> usize {
        match self.reg_bits {
            0b010 => self.not(),
            0b011 => self.neg(),
            0b100 => self.mul(),
            0b101 => self.imul(),
            0b110 => self.div(),
            0b111 => self.idiv(),
            _ => 0
        }
    }

    pub fn add(&mut self) -> usize {
        self.check_carry_add(self.src.clone().unwrap());
        let sum = self.operation_2_args(|src, dst| Self::add_with_carry_8_bit(dst, src), |src, dst| Self::add_with_carry_16_bit(dst, src));
        self.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
        self.write_to_arg(self.dst.clone().unwrap(), sum).unwrap();
        0
    }

    pub fn sub(&mut self) -> usize {
        self.check_carry_sub(self.src.clone().unwrap());
        let dif = self.operation_2_args(|src, dst| Self::sub_with_carry_8_bit(dst, src), |src, dst| Self::sub_with_carry_16_bit(dst, src));
        self.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
        self.write_to_arg(self.dst.clone().unwrap(), dif).unwrap();
        0
    }

    pub fn and(&mut self) -> usize {
        let result = self.operation_2_args(|src, dst| dst & src, |src, dst| dst & src);
        self.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }

    pub fn or(&mut self) -> usize {
        let result = self.operation_2_args(|src, dst| dst | src, |src, dst| dst | src);
        self.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }

    pub fn xor(&mut self) -> usize {
        let result = self.operation_2_args(|src, dst| dst ^ src, |src, dst| dst ^ src);
        self.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }

    pub fn not(&mut self) -> usize {
        let result = self.operation_1_arg(|dst| !dst, |dst| !dst);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }

    pub fn neg(&mut self) -> usize {
        let result = self.operation_1_arg(|dst| Self::twos_compliment_byte(dst), |dst| Self::twos_compliment_word(dst));
        self.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }

    pub fn inc(&mut self) -> usize {
        // self.check_carry_add(SrcArg::Byte(1));
        match self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Byte(dst) => self.set_flag_if(CPUFlags::OVERFLOW, dst as u16 + 1 > 255),
            SrcArg::Word(dst) => self.set_flag_if(CPUFlags::OVERFLOW, dst as u32 + 1 > 65535)

        }
        let sum = self.operation_1_arg(|dst| {
            Self::add_with_carry_8_bit(dst, 1)
        }, |dst| {
            Self::add_with_carry_16_bit(dst, 1)
        });
        self.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
        self.write_to_arg(self.dst.clone().unwrap(), sum).unwrap();
        0
    }

    pub fn dec(&mut self) -> usize {
        let sum = self.operation_1_arg(|dst| Self::sub_with_carry_8_bit(dst, 1), |dst| Self::sub_with_carry_16_bit(dst, 1));
        self.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
        self.write_to_arg(self.dst.clone().unwrap(), sum).unwrap();
        0
    }

    fn set_overflow(&mut self, result_high: u16) {
        if result_high == 0 {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::CARRY | CPUFlags::OVERFLOW;
        } else {
            self.regs.get_mut(&Regs::FLAGS).unwrap().value &= !(CPUFlags::CARRY | CPUFlags::OVERFLOW);
        }
    }

    pub fn mul(&mut self) -> usize {
        let operand = self.regs.get(&Regs::AX).unwrap().value;
        let (result_low, result_high, is_word) = match self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Byte(val) => {
                ((val as u16) * (operand & 0xFF), 0, false)
            }
            SrcArg::Word(val) => {
                (((val as u32) * (operand as u32)) as u16, (((val as u32) * (operand as u32)) >> 16) as u16, false)
            }
        };
        self.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_low)).unwrap();
        if is_word {
            self.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_high)).unwrap();
            self.set_overflow(result_high)
        }
        0
    }

    pub fn imul(&mut self) -> usize {
        let operand = self.regs.get(&Regs::AX).unwrap().value as i16;
        let (result_low, result_high, is_word) = match self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Byte(val) => {
                ((val as i16) * (operand & 0xFF), 0, false)
            }
            SrcArg::Word(val) => {
                (((val as i32) * (operand as i32)) as i16, (((val as i32) * (operand as i32)) >> 16) as i16, false)
            }
        };
        self.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_low as u16)).unwrap();
        if is_word {
            self.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_high as u16)).unwrap();
            self.set_overflow(result_high as u16);
        }
        0
    }

    pub fn div(&mut self) -> usize {
        match self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Byte(val) => {
                let operand = self.get_reg_16(0).unwrap();
                let result_div = (operand / (val as u16)) as u8;
                let result_mod = (operand % (val as u16)) as u8;
                let result = SrcArg::Word((result_div as u16) | ((result_mod as u16) << 8));
                self.write_to_arg(DstArg::Reg16(0), result).unwrap();
            },
            SrcArg::Word(val) => {
                let operand = (self.get_reg_16(0).unwrap() as u32) | ((self.get_reg_16(2).unwrap() as u32) << 16);
                let result_div = (operand / (val as u32)) as u16;
                let result_mod = (operand % (val as u32)) as u16;
                self.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_div)).unwrap();
                self.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_mod)).unwrap();
            }
        };
        0
    }

    pub fn idiv(&mut self) -> usize {
        match self.get_src_arg(self.dst.clone().unwrap()).unwrap() {
            SrcArg::Byte(val) => {
                let operand = self.get_reg_16(0).unwrap() as i16;
                let result_div = (operand / (val as i16)) as i8;
                let result_mod = (operand % (val as i16)) as i8;
                let result = SrcArg::Word(((result_div as i16) | ((result_mod as i16) << 8)) as u16);
                self.write_to_arg(DstArg::Reg16(0), result).unwrap();
            },
            SrcArg::Word(val) => {
                let operand = (self.get_reg_16(0).unwrap() as i32) | ((self.get_reg_16(2).unwrap() as i32) << 16);
                let result_div = (operand / (val as i32)) as u16;
                let result_mod = (operand % (val as i32)) as u16;
                self.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_div)).unwrap();
                self.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_mod)).unwrap();
            }
        };
        0
    }
}
