use super::{CPU};
use crate::cpu::SrcArg;

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
            _ => 0
        }
    }

    pub fn add(&mut self) -> usize {
        self.check_carry_add(self.src.clone().unwrap());
        let sum = self.operation_2_args(|src, dst| Self::add_with_carry_8_bit(dst, src), |src, dst| Self::add_with_carry_16_bit(dst, src));
        self.check_zero(&sum);
        self.write_to_arg(self.dst.clone().unwrap(), sum).unwrap();
        0
    }

    pub fn sub(&mut self) -> usize {
        self.check_carry_sub(self.src.clone().unwrap());
        let dif = self.operation_2_args(|src, dst| Self::sub_with_carry_8_bit(dst, src), |src, dst| Self::sub_with_carry_16_bit(dst, src));
        self.check_zero(&dif);
        self.write_to_arg(self.dst.clone().unwrap(), dif).unwrap();
        0
    }

    pub fn and(&mut self) -> usize {
        let result = self.operation_2_args(|src, dst| dst & src, |src, dst| dst & src);
        self.check_zero(&result);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }

    pub fn or(&mut self) -> usize {
        let result = self.operation_2_args(|src, dst| dst | src, |src, dst| dst | src);
        self.check_zero(&result);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }

    pub fn xor(&mut self) -> usize {
        let result = self.operation_2_args(|src, dst| dst ^ src, |src, dst| dst ^ src);
        self.check_zero(&result);
        self.write_to_arg(self.dst.clone().unwrap(), result).unwrap();
        0
    }


    pub fn inc(&mut self) -> usize {
        self.check_carry_add(SrcArg::Byte(1));
        let sum = self.operation_1_arg(|dst| Self::add_with_carry_8_bit(dst, 1), |dst| Self::add_with_carry_16_bit(dst, 1));
        self.check_zero(&sum);
        self.write_to_arg(self.dst.clone().unwrap(), sum).unwrap();
        0
    }

    pub fn dec(&mut self) -> usize {
        self.check_carry_sub(SrcArg::Byte(1));
        let sum = self.operation_1_arg(|dst| Self::sub_with_carry_8_bit(dst, 1), |dst| Self::sub_with_carry_16_bit(dst, 1));
        self.check_zero(&sum);
        self.write_to_arg(self.dst.clone().unwrap(), sum).unwrap();
        0
    }
}
