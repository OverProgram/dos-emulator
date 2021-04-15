use crate::cpu::instruction::actions::{stack, jmp};
use crate::cpu::{CPU, Regs, CPUFlags, exceptions};
use crate::cpu::instruction::actions::flags::{cmp, test};
use crate::cpu::instruction::args::{SrcArg, DstArg};
use crate::cpu::instruction::Instruction;

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

fn rotate_left_byte(arg: u8, times: u8) -> u8 {
    let last = arg >> (8 - times);
    (arg << times) | last
}

fn rotate_left_word(arg: u16, times: u16) -> u16 {
    let last = arg >> (16 - times);
    (arg << times) | last
}

fn rotate_right_byte(arg: u8, times: u8) -> u8 {
    let mut mask = 0;
    for _ in 0..times {
        mask <<= 1;
        mask |= 1;
    }
    let first = arg & mask;
    (arg >> times) | (first << (8 - times))
}

fn rotate_right_word(arg: u16, times: u16) -> u16 {
    let mut mask = 0;
    for _ in 0..times {
        mask <<= 1;
        mask |= 1;
    }
    let first = arg & mask;
    (arg >> times) | (first << (16 - times))
}

fn rotate_left_carry_byte(arg: u8, times: u8, carry: u8) -> (u8, u8) {
    let mut num = arg;
    let mut new_carry = carry;
    for _ in 0..times {
        let tmp_carry = new_carry;
        new_carry = (num & 0x01) as u8;
        num = (num << 1) | (tmp_carry << 7);
    }
    (num, new_carry)
}

fn rotate_left_carry_word(arg: u16, times: u16, carry: u8) -> (u16, u8) {
    let mut num = arg;
    let mut new_carry = carry;
    for _ in 0..times {
        let tmp_carry = new_carry;
        new_carry = (num & 0x01) as u8;
        num = (num << 1) | ((tmp_carry as u16) << 15);
    }
    (num, new_carry)
}

fn rotate_right_carry_byte(arg: u8, times: u8, carry: u8) -> (u8, u8) {
    let mut num = arg;
    let mut new_carry = carry;
    for _ in 0..times {
        let tmp_carry = new_carry;
        new_carry = (num & 0x01) as u8;
        num = (num >> 1) | (tmp_carry << 7);
    }
    (num, new_carry)
}

fn rotate_right_carry_word(arg: u16, times: u16, carry: u8) -> (u16, u8) {
    let mut num = arg;
    let mut new_carry = carry;
    for _ in 0..times {
        let tmp_carry = new_carry;
        new_carry = (num & 0x01) as u8;
        num = (num >> 1) | ((tmp_carry as u16) << 15);
    }
    (num, new_carry)
}

pub fn alu_dispatch_two_args(comp: &mut CPU, instruction: Instruction) -> usize {
    match comp.instruction.clone().unwrap().reg_bits {
        0b000 => add(comp, instruction),
        0b001 => or(comp, instruction),
        0b010 => adc(comp, instruction),
        0b011 => sbb(comp, instruction),
        0b100 => and(comp, instruction),
        0b101 => sub(comp, instruction),
        0b110 => xor(comp, instruction),
        0b111 => cmp(comp, instruction),
        _ => 0
    }
}

pub fn alu_dispatch_two_args_mnemonic(instruction: Instruction) -> String {
    String::from(match instruction.reg_bits {
        0b000 => "add",
        0b001 => "or",
        0b010 => "adc",
        0b011 => "sbb",
        0b100 => "and",
        0b101 => "sub",
        0b110 => "xor",
        _ => panic!("invalid reg_bits value")
    })
}

pub fn alu_dispatch_one_arg(comp: &mut CPU, instruction: Instruction) -> usize {
    match comp.instruction.clone().unwrap().reg_bits {
        0b000 => inc(comp, instruction),
        0b001 => dec(comp, instruction),
        0b010 => stack::near_call(comp, instruction),
        0b011 => stack::far_call(comp, instruction),
        0b100 => jmp::jmp(comp, instruction),
        0b101 => jmp::jmp_far(comp, instruction),
        0b110 => stack::push(comp, instruction),
        _ => 0
    }
}

pub fn alu_dispatch_one_arg_mnemonic(instruction: Instruction) -> String {
    String::from(match instruction.reg_bits {
        0b000 => "inc",
        0b001 => "dec",
        0b010 | 0b011 => "call",
        0b100 | 0b101 => "jmp",
        0b110 => "push",
        _ => panic!("invalid reg_bits value")
    })
}

pub fn mul_dispatch(comp: &mut CPU, instruction: Instruction) -> usize {
    match comp.instruction.clone().unwrap().reg_bits {
        0b000 => test(comp, instruction),
        0b010 => not(comp, instruction),
        0b011 => neg(comp, instruction),
        0b100 => mul(comp, instruction),
        0b101 => imul(comp, instruction),
        0b110 => div(comp, instruction),
        0b111 => idiv(comp, instruction),
        _ => 0
    }
}

pub fn mul_dispatch_mnemonic(instruction: Instruction) -> String {
    String::from(match instruction.reg_bits {
        0b000 => "test",
        0b010 => "not",
        0b011 => "neg",
        0b100 => "mul",
        0b101 => "imul",
        0b110 => "div",
        0b111 => "idiv",
        _ => panic!("invalid reg_bits value")
    })
}

pub fn rotate_dispatch(comp: &mut CPU, instruction: Instruction) -> usize {
    match instruction.reg_bits {
        0b000 => rol(comp, instruction),
        0b001 => ror(comp, instruction),
        0b010 => rcl(comp, instruction),
        0b011 => rcr(comp, instruction),
        0b100 => sal(comp, instruction),
        0b101 => shr(comp, instruction),
        0b111 => sar(comp, instruction),
        _ => 0
    }
}

pub fn rotate_dispatch_mnemonic(instruction: Instruction) -> String {
    match instruction.reg_bits {
        0b000 => "rol",
        0b001 => "ror",
        0b010 => "rcl",
        0b011 => "rcr",
        0b100 => "sal",
        0b101 => "shr",
        0b111 => "sar",
        _ => ""
    }.to_string()
}

pub fn add(comp: &mut CPU, instruction: Instruction) -> usize {
    let src = instruction.src.clone().unwrap().to_src_arg(comp).unwrap();
    comp.check_carry_add(src);
    let sum = comp.operation_2_args(|src, dst| add_with_carry_8_bit(dst, src), |src, dst| add_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
    comp.write_to_arg(instruction.dst.clone().unwrap(), sum).unwrap();
    0
}

pub fn adc(comp: &mut CPU, instruction: Instruction) -> usize {
    let cf = if comp.check_flag(CPUFlags::CARRY) { 1 } else { 0 };
    let src = comp.operation_2_args(|src, _| src + (cf as u8), |src, _| src + cf);
    comp.check_carry_add(src.clone());
    let sum = match src {
        SrcArg::Word(val) => {
            match instruction.dst.unwrap().to_src_arg(comp).unwrap() {
                SrcArg::Word(dst) => Some(SrcArg::Word(val + dst)),
                _ => None
            }
        }
        SrcArg::Byte(val) => {
            match instruction.dst.unwrap().to_src_arg(comp).unwrap() {
                SrcArg::Byte(dst) => Some(SrcArg::Byte(val + dst)),
                _ => None,
            }
        },
        _ => None
    }.unwrap();
    comp.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(instruction.dst.clone().unwrap(), sum).unwrap();
    0
}

pub fn sub(comp: &mut CPU, instruction: Instruction) -> usize {
    let src = instruction.src.clone().unwrap().to_src_arg(comp).unwrap();
    comp.check_carry_sub(src);
    let dif = comp.operation_2_args(|src, dst| sub_with_carry_8_bit(dst, src), |src, dst| sub_with_carry_16_bit(dst, src));
    comp.check_flags_in_result(&dif, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(instruction.dst.clone().unwrap(), dif).unwrap();
    0
}

pub fn sbb(comp: &mut CPU, instruction: Instruction) -> usize {
    let cf = if comp.check_flag(CPUFlags::CARRY) { 1u8 } else { 0u8 };
    let src = comp.operation_2_args(|src, _| src + cf, |src, _| src + (cf as u16));
    comp.check_carry_sub(src);
    let res = comp.operation_2_args(|src, dst| dst - (src + cf), |src, dst| dst - (src + (cf as u16)));
    comp.check_flags_in_result(&res, CPUFlags::PARITY | CPUFlags::ZERO | CPUFlags::SIGN | CPUFlags::AUX_CARRY);
    comp.write_to_arg(instruction.dst.unwrap(), res).unwrap();
    0
}

pub fn and(comp: &mut CPU, instruction: Instruction) -> usize {
    let result = comp.operation_2_args(|src, dst| dst & src, |src, dst| dst & src);
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
    comp.clear_flag(CPUFlags::OVERFLOW | CPUFlags::CARRY);
    comp.write_to_arg(instruction.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn or(comp: &mut CPU, instruction: Instruction) -> usize {
    let result = comp.operation_2_args(|src, dst| dst | src, |src, dst| dst | src);
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
    comp.clear_flag(CPUFlags::OVERFLOW | CPUFlags::CARRY);
    comp.write_to_arg(instruction.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn xor(comp: &mut CPU, instruction: Instruction) -> usize {
    let result = comp.operation_2_args(|src, dst| dst ^ src, |src, dst| dst ^ src);
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO);
    comp.clear_flag(CPUFlags::OVERFLOW | CPUFlags::CARRY);
    comp.write_to_arg(instruction.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn not(comp: &mut CPU, instruction: Instruction) -> usize {
    let result = comp.operation_1_arg(|dst| !dst, |dst| !dst);
    comp.write_to_arg(instruction.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn neg(comp: &mut CPU, instruction: Instruction) -> usize {
    let result = comp.operation_1_arg(|dst| twos_compliment_byte(dst), |dst| twos_compliment_word(dst));
    comp.check_flags_in_result(&result, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(instruction.dst.clone().unwrap(), result).unwrap();
    0
}

pub fn inc(comp: &mut CPU, instruction: Instruction) -> usize {
    // comp.check_carry_add(SrcArg::Byte(1));
    match instruction.dst.clone().unwrap().to_src_arg(comp).unwrap() {
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
    comp.write_to_arg(instruction.dst.clone().unwrap(), sum).unwrap();
    0
}

pub fn dec(comp: &mut CPU, instruction: Instruction) -> usize {
    let sum = comp.operation_1_arg(|dst| sub_with_carry_8_bit(dst, 1), |dst| sub_with_carry_16_bit(dst, 1));
    comp.check_flags_in_result(&sum, CPUFlags::PARITY | CPUFlags::SIGN | CPUFlags::ZERO | CPUFlags::AUX_CARRY);
    comp.write_to_arg(instruction.dst.clone().unwrap(), sum).unwrap();
    0
}

fn set_overflow(comp: &mut CPU, result_high: u16) {
    if result_high == 0 {
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::CARRY | CPUFlags::OVERFLOW;
    } else {
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value &= !(CPUFlags::CARRY | CPUFlags::OVERFLOW);
    }
}

pub fn mul(comp: &mut CPU, instruction: Instruction) -> usize {
    let operand = comp.regs.get(&Regs::AX).unwrap().value;
    let (result_low, result_high, is_word) = match instruction.dst.clone().unwrap().to_src_arg(comp).unwrap() {
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

pub fn imul(comp: &mut CPU, instruction: Instruction) -> usize {
    let operand = comp.regs.get(&Regs::AX).unwrap().value as i16;
    let (result_low, result_high, is_word) = match instruction.dst.clone().unwrap().to_src_arg(comp).unwrap() {
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

pub fn div(comp: &mut CPU, instruction: Instruction) -> usize {
    match instruction.dst.clone().unwrap().to_src_arg(comp).unwrap() {
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

pub fn idiv(comp: &mut CPU, instruction: Instruction) -> usize {
    match instruction.dst.clone().unwrap().to_src_arg(comp).unwrap() {
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

pub fn aaa(comp: &mut CPU, _: Instruction) -> usize {
    let al = comp.get_reg_8(0).unwrap();
    if al & 0x0F > 9 || comp.check_flag(CPUFlags::AUX_CARRY) {
        let ax = comp.regs.get_mut(&Regs::AX).unwrap();
        let (ax_high, ax_low) = (ax.get_high(), ax.get_low());
        ax.set_high(ax_high + 1);
        ax.set_low(ax_low + 6);
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value |= CPUFlags::AUX_CARRY | CPUFlags::CARRY;
    } else {
        comp.regs.get_mut(&Regs::FLAGS).unwrap().value &= !(CPUFlags::AUX_CARRY | CPUFlags::CARRY);
    }
    let new_al = comp.get_reg_8(0).unwrap() & 0x0F;
    comp.regs.get_mut(&Regs::AX).unwrap().set_low(new_al);
    0
}

pub fn aad(comp: &mut CPU, instruction: Instruction) -> usize {
    if let Some(DstArg::Imm8(base)) = instruction.dst {
        let ax = comp.regs.get_mut(&Regs::AX).unwrap();
        let al = ax.get_low();
        let ah = ax.get_high();

        ax.set_low((al + (ah * base)) & 0xFF);
        ax.set_high(0x00);
    }
    0
}

pub fn aas(comp: &mut CPU, _: Instruction) -> usize {
    let al = comp.get_reg_8(0).unwrap();

    if (al & 0xF) > 9 || comp.check_flag(CPUFlags::AUX_CARRY) {
        let ax = comp.regs.get_mut(&Regs::AX).unwrap();
        let ah = ax.get_high();
        ax.value -= 6;
        ax.set_high(ah - 1);
        let al = ax.get_low();
        ax.set_low(al & 0x0F);
        comp.set_flag(CPUFlags::AUX_CARRY);
        comp.set_flag(CPUFlags::CARRY);
    } else {
        comp.clear_flag(CPUFlags::AUX_CARRY);
        comp.clear_flag(CPUFlags::CARRY);
    }
    0
}

pub fn daa(comp: &mut CPU, _: Instruction) -> usize {
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

pub fn ror(comp: &mut CPU, instruction: Instruction) -> usize {
    let res = comp.operation_2_args(|src, dst| rotate_right_byte(dst, src), |src, dst| rotate_right_word(dst, src));
    comp.write_to_arg(*instruction.dst.as_ref().unwrap(), res).unwrap();
    0
}

pub fn rol(comp: &mut CPU, instruction: Instruction) -> usize {
    let res = comp.operation_2_args(|src, dst| rotate_left_byte(dst, src), |src, dst| rotate_left_word(dst, src));
    comp.write_to_arg(*instruction.dst.as_ref().unwrap(), res).unwrap();
    0
}

fn get_times(src: SrcArg) -> u8 {
   match src {
       SrcArg::Byte(val) => val,
       SrcArg::Word(val) => val as u8,
       SrcArg::DWord(val) => val as u8
   }
}

pub fn rcr(comp: &mut CPU, instruction: Instruction) -> usize {
    let carry = if comp.check_flag(CPUFlags::CARRY) { 1 } else { 0 };
    let times = get_times(instruction.src.as_ref().unwrap().to_src_arg(comp).unwrap());

    let src = match instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap() {
        SrcArg::Byte(dst) => {
            let (new_src, new_carry) = rotate_right_carry_byte(dst, times, carry);
            if new_carry & 0x01 == 1 {
                comp.set_flag(CPUFlags::CARRY);
            } else {
                comp.clear_flag(CPUFlags::CARRY);
            }
            SrcArg::Byte(new_src)
        }
        SrcArg::Word(dst) => {
            let (new_src, new_carry) = rotate_right_carry_word(dst, times as u16, carry);
            if new_carry & 0x01 == 1 {
                comp.set_flag(CPUFlags::CARRY);
            } else {
                comp.clear_flag(CPUFlags::CARRY);
            }
            SrcArg::Word(new_src)
        }
        _ => panic!("rcr only accepts byte or word")
    };

    comp.write_to_arg(*instruction.dst.as_ref().unwrap(), src).unwrap();
    0
}

pub fn rcl(comp: &mut CPU, instruction: Instruction) -> usize {
    let carry = if comp.check_flag(CPUFlags::CARRY) { 1 } else { 0 };
    let times = get_times(instruction.src.as_ref().unwrap().to_src_arg(comp).unwrap());

    let src = match instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap() {
        SrcArg::Byte(dst) => {
            let (new_src, new_carry) = rotate_left_carry_byte(dst, times, carry);
            if new_carry & 0x01 == 1 {
                comp.set_flag(CPUFlags::CARRY);
            } else {
                comp.clear_flag(CPUFlags::CARRY);
            }
            SrcArg::Byte(new_src)
        }
        SrcArg::Word(dst) => {
            let (new_src, new_carry) = rotate_left_carry_word(dst, times as u16, carry);
            if new_carry & 0x01 == 1 {
                comp.set_flag(CPUFlags::CARRY);
            } else {
                comp.clear_flag(CPUFlags::CARRY);
            }
            SrcArg::Word(new_src)
        }
        _ => panic!("rcl only accepts byte or word")
    };

    comp.write_to_arg(*instruction.dst.as_ref().unwrap(), src).unwrap();
    0
}

fn shift_check_carry(comp: &mut CPU, instruction: &Instruction, times: u8, left: bool) {
    let mut mask = if left { 0x01 } else { 0x80 };

    for _ in 1..times {
        if left {
            mask <<= 1;
        } else {
            mask >>= 1;
        }
    }

    if CPU::check_src_arg(&instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap(),
                          |dst| dst & mask != 0,
                          |dst| dst & (mask as u16) != 0) {
        comp.set_flag(CPUFlags::CARRY);
    } else {
        comp.clear_flag(CPUFlags::CARRY);
    }
}
fn shift_get_times(comp: &mut CPU, instruction: &Instruction) -> u8 {
    match instruction.src.as_ref().unwrap().to_src_arg(comp).unwrap() {
        SrcArg::Byte(val) => val,
        _ => panic!("shift operation is only allowed byte as src arg")
    }
}

pub fn sal(comp: &mut CPU, instruction: Instruction) -> usize {
    let times = match instruction.src.unwrap().to_src_arg(comp).unwrap() {
        SrcArg::Byte(times) => if times == 1 {
            if CPU::check_src_arg(&instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap(),
                                  |dst| ((dst & 0x80) >> 7) == ((dst & 0x40) >> 6),
                                  |dst| ((dst & 0x8000) >> 15) == ((dst & 0x4000) >> 14)) {
                comp.set_flag(CPUFlags::OVERFLOW);
            } else {
                comp.clear_flag(CPUFlags::OVERFLOW);
            }
            times
        } else {
            times
        }
        _ => panic!("sal can only get a byte arg for times")
    };

    let res = comp.operation_2_args(|src, dst| dst << src, |src, dst| dst << src);

    shift_check_carry(comp, &instruction, times, true);

    comp.write_to_arg(instruction.dst.unwrap(), res).unwrap();

    0
}

pub fn shr(comp: &mut CPU, instruction: Instruction) -> usize {
    comp.set_flag(CPUFlags::OVERFLOW);

    let res = comp.operation_2_args(|src, dst| dst >> src, |dst, src| dst >> src);

    let times = shift_get_times(comp, &instruction);
    shift_check_carry(comp, &instruction, times, false);

    comp.write_to_arg(instruction.dst.unwrap(), res).unwrap();

    0
}

fn arithmetic_right_shift_word(times: u16, arg: u16) -> u16 {
    ((arg & (0x7FFF)) >> times) | (arg & 0x8000)
}

fn arithmetic_right_shift_byte(times: u8, arg: u8) -> u8 {
    ((arg & (0x7F)) >> times) | (arg & 0x80)
}

pub fn sar(comp: &mut CPU, instruction: Instruction) -> usize {
    if CPU::check_src_arg(&instruction.dst.as_ref().unwrap().to_src_arg(comp).unwrap(),
                          |dst| dst & 0x80 != 0, |dst| dst & 0x8000 != 0) {
        comp.set_flag(CPUFlags::OVERFLOW);
    } else {
        comp.clear_flag(CPUFlags::OVERFLOW);
    };

    let res = comp.operation_2_args(arithmetic_right_shift_byte, arithmetic_right_shift_word);

    let times = shift_get_times(comp, &instruction);
    shift_check_carry(comp, &instruction, times, false);

    comp.write_to_arg(instruction.dst.unwrap(), res).unwrap();

    0
}
