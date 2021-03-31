use super::{CPU};
use crate::cpu::{SrcArg, DstArg, Regs, CPUFlags, stack, exceptions, jmp};
use crate::cpu::flags::cmp;

pub fn add_with_carry_16_bit(arg1: u16, arg2: u16) -> u16 {
    let sum = ((arg1 as u32) + (arg2 as u32)) % 65536;
    sum as u16
}

pub fn add_with_carry_8_bit(arg1: u8, arg2: u8) -> u8 {
    let sum = ((arg1 as u16) + (arg2 as u16)) % 256;
    sum as u8
}

pub fn sub_with_carry_16_bit(arg1: u16, arg2: u16) -> u16 {
    let mut sum = (arg1 as i32) - (arg2 as i32);
    if sum < 0 {
        sum += 65536;
    }
    sum as u16
}

pub fn sub_with_carry_8_bit( arg1: u8, arg2: u8) -> u8 {
    let mut sum = (arg1 as i16) - (arg2 as i16);
    if sum < 0 {
        sum += 256;
    }
    sum as u8
}

pub fn twos_compliment_word(arg: u16) -> u16 {
    add_with_carry_16_bit(!arg, 1)
}

pub fn twos_compliment_byte(arg: u8) -> u8 {
    add_with_carry_8_bit(!arg, 1)
}

pub fn alu_dispatch_two_args(comp: &mut CPU) -> usize {
    match comp.reg_bits {
        0b000 => add(comp),
        0b001 => or(comp),
        0b010 => adc(comp),
        0b100 => and(comp),
        0b101 => sub(comp),
        0b110 => xor(comp),
        0b111 => cmp(comp),
        _ => 0
    }
}

pub fn alu_dispatch_two_args_mnemonic(reg_bits: u8) -> Option<String> {
    Some(String::from(match reg_bits {
        0b000 => "ADD",
        0b001 => "OR",
        0b010 => "ADC",
        0b100 => "AND",
        0b101 => "SUB",
        0b110 => "XOR",
        _ => return None
    }))
}

pub fn alu_dispatch_one_arg(comp: &mut CPU) -> usize {
    match comp.reg_bits {
        0b000 => inc(comp),
        0b001 => dec(comp),
        0b010 => stack::near_call(comp),
        0b011 => stack::far_call(comp),
        0b100 => jmp::jmp(comp),
        0b101 => jmp::jmp_far(comp),
        0b110 => stack::push(comp),
        _ => 0
    }
}

pub fn alu_dispatch_one_arg_mnemonic(reg_bits: u8) -> Option<String> {
    Some(String::from(match reg_bits {
        0b000 => "INC",
        0b001 => "DEC",
        0b010 | 0b011 => "CALL",
        0b100 | 0b101 => "JMP",
        0b110 => "PUSH",
        _ => return None
    }))
}

pub fn mul_dispatch(comp: &mut CPU) -> usize {
    match comp.reg_bits {
        0b010 => not(comp),
        0b011 => neg(comp),
        0b100 => mul(comp),
        0b101 => imul(comp),
        0b110 => div(comp),
        0b111 => idiv(comp),
        _ => 0
    }
}

pub fn mul_dispatch_mnemonic(reg_bits: u8) -> Option<String> {
    Some(String::from(match reg_bits {
        0b010 => "NOT",
        0b011 => "NEG",
        0b100 => "MUL",
        0b101 => "IMUL",
        0b110 => "DIV",
        0b111 => "IDIV",
        _ => return None
    }))
}

pub fn add(comp: &mut CPU) -> usize {
    comp.check_carry_add(comp.src.clone().unwrap());
    let sum = comp.operation_2_args(|src, dst| add_with_carry_8_bit(dst, src), |src, dst| add_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(comp.dst.clone().unwrap(), sum).unwrap();
    0
}

pub fn add_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("ADD"))
                                                                     }

pub fn adc(comp: &mut CPU) -> usize {
    let cf = (comp.read_reg(Regs::FLAGS).unwrap() & CPUFlags::CARRY) >> 0x01;
    let src = comp.operation_2_args(|src, _| src + (cf as u8), |src, _| src + cf);
    comp.check_carry_add(src.clone());
    let sum = match src {
        SrcArg::Word(val) => {
            match comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap() {
                SrcArg::Word(dst) => Some(SrcArg::Word(val + dst)),
                _ => None
            }
        }
        SrcArg::Byte(val) => {
            match comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap() {
                SrcArg::Byte(dst) => Some(SrcArg::Byte(val + dst)),
                _ => None,
            }
        },
        _ => None
    }.unwrap();
    comp.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(comp.dst.clone().unwrap(), sum).unwrap();
    0
}

pub fn adc_mnemonic(_: u8) -> Option<String> {
    Some(String::from("ADC"))
}

pub fn sub(comp: &mut CPU) -> usize {
    comp.check_carry_sub(comp.src.clone().unwrap());
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(comp.dst.clone().unwrap(), dif).unwrap();
    0
}

pub fn sub_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("SUB"))
                                                                     }

pub fn and(comp: &mut CPU) -> usize {
    let result = comp.operation_2_args(|src, dst| dst & src, |src, dst| dst & src);
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
    comp.write_to_arg(comp.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn and_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("AND"))
                                                                     }

pub fn or(comp: &mut CPU) -> usize {
    let result = comp.operation_2_args(|src, dst| dst | src, |src, dst| dst | src);
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
    comp.write_to_arg(comp.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn or_mnemonic(_: u8) -> Option<String> {
                                          Some(String::from("OR"))
                                                                   }

pub fn xor(comp: &mut CPU) -> usize {
    let result = comp.operation_2_args(|src, dst| dst ^ src, |src, dst| dst ^ src);
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
    comp.write_to_arg(comp.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn xor_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("XOR"))
                                                                     }

pub fn not(comp: &mut CPU) -> usize {
    let result = comp.operation_1_arg(|dst| !dst, |dst| !dst);
    comp.write_to_arg(comp.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn neg(comp: &mut CPU) -> usize {
    let result = comp.operation_1_arg(|dst| twos_compliment_byte(dst), |dst| twos_compliment_word(dst));
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(comp.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn inc(comp: &mut CPU) -> usize {
    // comp.check_carry_add(SrcArg::Byte(1));
    match comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap() {
        SrcArg::Byte(dst) => comp.set_flag_if(CPUFlags::OVERFLOW, dst as u16 + 1 > 255),
        SrcArg::Word(dst) => comp.set_flag_if(CPUFlags::OVERFLOW, dst as u32 + 1 > 65535),
        _ => ()
    }
    let sum = comp.operation_1_arg(|dst| {
        add_with_carry_8_bit(dst, 1)
    }, |dst| {
        add_with_carry_16_bit(dst, 1)
    });
    comp.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(comp.dst.clone().unwrap(), sum).unwrap();
    0
}

pub fn inc_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("INC"))
                                                                     }

pub fn dec(comp: &mut CPU) -> usize {
    let sum = comp.operation_1_arg(|dst| sub_with_carry_8_bit(dst, 1), |dst| sub_with_carry_16_bit(dst, 1));
    comp.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(comp.dst.clone().unwrap(), sum).unwrap();
    0
}

pub fn dec_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("DEC"))
                                                                     }

fn set_overflow(comp: &mut CPU, result_high: u16) {
    if result_high == 0 {
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::CARRY | CPUFlags::OVERFLOW;
    } else {
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value &= !(CPUFlags::CARRY | CPUFlags::OVERFLOW);
    }
}

pub fn mul(comp: &mut CPU) -> usize {
    let operand = comp.regs.get(&Regs::AX).unwrap().value;
    let (result_low, result_high, is_word) = match comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap() {
        SrcArg::Byte(val) => {
            ((val as u16) * (operand & 0xFF), 0, false)
        }
        SrcArg::Word(val) => {
            (((val as u32) * (operand as u32)) as u16, (((val as u32) * (operand as u32)) >> 16) as u16, false)
        }
        SrcArg::DWord(_) => {
            panic!("Can't use DWord SrcArg in this opcode");
        }
    };
    comp.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_low)).unwrap();
    if is_word {
        comp.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_high)).unwrap();
        set_overflow(comp, result_high)
    }
    0
}

pub fn imul(comp: &mut CPU) -> usize {
    let operand = comp.regs.get(&Regs::AX).unwrap().value as i16;
    let (result_low, result_high, is_word) = match comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap() {
        SrcArg::Byte(val) => {
            ((val as i16) * (operand & 0xFF), 0, false)
        }
        SrcArg::Word(val) => {
            (((val as i32) * (operand as i32)) as i16, (((val as i32) * (operand as i32)) >> 16) as i16, false)
        }
        SrcArg::DWord(_) => {
            panic!("Can't use DWord SrcArg in this opcode");
        }
    };
    comp.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_low as u16)).unwrap();
    if is_word {
        comp.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_high as u16)).unwrap();
        set_overflow(comp, result_high as u16);
    }
    0
}

pub fn div(comp: &mut CPU) -> usize {
    match comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap() {
        SrcArg::Byte(val) => {
            if val == 0 {
                comp.except(exceptions::DIVIDE_BY_ZERO).unwrap();
            } else {
                let operand = comp.get_reg_16(0).unwrap();
                let result_div = (operand / (val as u16)) as u8;
                let result_mod = (operand % (val as u16)) as u8;
                let result = SrcArg::Word((result_div as u16) | ((result_mod as u16) << 8));
                comp.write_to_arg(DstArg::Reg16(0), result).unwrap();
            }
        },
        SrcArg::Word(val) => {
            if val == 0 {
                comp.except(exceptions::DIVIDE_BY_ZERO).unwrap();
            } else {
                let operand = (comp.get_reg_16(0).unwrap() as u32) | ((comp.get_reg_16(2).unwrap() as u32) << 16);
                let result_div = (operand / (val as u32)) as u16;
                let result_mod = (operand % (val as u32)) as u16;
                comp.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_div)).unwrap();
                comp.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_mod)).unwrap();
            }
        }
        SrcArg::DWord(_) => {
            panic!("Can't use DWord SrcArg in this opcode");
        }
    };
    0
}

pub fn idiv(comp: &mut CPU) -> usize {
    match comp.get_src_arg_mut(comp.dst.clone().unwrap()).unwrap() {
        SrcArg::Byte(val) => {
            if val == 0 {
                comp.except(exceptions::DIVIDE_BY_ZERO).unwrap();
            } else {
                let operand = comp.get_reg_16(0).unwrap() as i16;
                let result_div = (operand / (val as i16)) as i8;
                let result_mod = (operand % (val as i16)) as i8;
                let result = SrcArg::Word(((result_div as i16) | ((result_mod as i16) << 8)) as u16);
                comp.write_to_arg(DstArg::Reg16(0), result).unwrap();
            }
        },
        SrcArg::Word(val) => {
            if val == 0 {
                comp.except(exceptions::DIVIDE_BY_ZERO).unwrap();
            } else {
                let operand = (comp.get_reg_16(0).unwrap() as i32) | ((comp.get_reg_16(2).unwrap() as i32) << 16);
                let result_div = (operand / (val as i32)) as u16;
                let result_mod = (operand % (val as i32)) as u16;
                comp.write_to_arg(DstArg::Reg16(0), SrcArg::Word(result_div)).unwrap();
                comp.write_to_arg(DstArg::Reg16(2), SrcArg::Word(result_mod)).unwrap();
            }
        }
        SrcArg::DWord(_) => {
            panic!("Can't use DWord SrcArg in this opcode");
        }
    };
    0
}

pub fn aaa(comp: &mut CPU) -> usize {
    let al = comp.get_reg_8(0).unwrap();
    if al & 0x0F > 9 || comp.regs[&Regs::FLAGS].value & CPUFlags::AUX_CARRY > 0 {
        let ax = comp.regs.get_mut(&Regs::AX).unwrap();
        let (ax_high, ax_low) = (ax.get_high(), ax.get_low());
        ax.set_high(ax_high + 1);
        ax.set_low(ax_low + 6);
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::AUX_CARRY | CPUFlags::CARRY;
    } else {
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value &= !(CPUFlags::AUX_CARRY | CPUFlags::CARRY);
    }
    0
}

pub fn aaa_mnemonic(_: u8) -> Option<String> {
                                           Some(String::from("AAA"))
                                                                     }

pub fn aad(comp: &mut CPU) -> usize {
    if let Some(DstArg::Imm8(base)) = comp.dst {
        let ax = comp.regs.get_mut(&Regs::AX).unwrap();
        let al = ax.get_low();
        let ah = ax.get_high();

        ax.set_low((al + (ah * base)) & 0xFF);
        ax.set_high(0x00);
    }
    0
}

pub fn aad_mnemonic(_: u8) -> Option<String> {
    Some(String::from("AAD"))
}

pub fn aas(comp: &mut CPU) -> usize {
    let al = comp.get_reg_8(0).unwrap();

    if (al & 0xF) > 9 || comp.check_flag(CPUFlags::AUX_CARRY) {
        let ax = comp.regs.get_mut(&Regs::AX).unwrap();
        let ah = ax.get_low();
        ax.set_low(al - 6);
        ax.set_high(ah - 1);
        comp.set_flag(CPUFlags::AUX_CARRY);
        comp.set_flag(CPUFlags::CARRY);
    } else {
        comp.clear_flag(CPUFlags::AUX_CARRY);
        comp.clear_flag(CPUFlags::CARRY);
    }
    0
}

pub fn aas_mnemonic(_: u8) -> Option<String> {
    Some(String::from("AAS"))
}

pub fn daa(comp: &mut CPU) -> usize {
    let old_al = comp.regs.get(&Regs::AX).unwrap().get_low();
    let old_cf = comp.check_flag(CPUFlags::CARRY);
    comp.clear_flag(CPUFlags::CARRY);
    if ((old_al & 0x0f) > 9) || comp.check_flag(CPUFlags::AUX_CARRY) {
        comp.regs.get_mut(&Regs::AX).unwrap().set_low(old_al.wrapping_add(6));
        if old_al.overflowing_add(6).1 || old_cf {
            comp.set_flag(CPUFlags::AUX_CARRY);
        } else {
           comp.clear_flag(CPUFlags::AUX_CARRY);
        }
    }
    if old_al > 0x99 || old_cf {
        let new_ax = comp.regs.get(&Regs::AX).unwrap().get_low().wrapping_add(0x60);
        comp.regs.get_mut(&Regs::AX).unwrap().set_low(new_ax);
        comp.set_flag(CPUFlags::CARRY);
    } else {
        comp.clear_flag(CPUFlags::CARRY);
    }
    0
}

pub fn daa_mnemonic(_: u8)-> Option<String> {
    Some(String::from("DAA"))
}
