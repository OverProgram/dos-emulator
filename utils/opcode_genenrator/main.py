import pathlib


class Function:
    def __init__(self, name, module):
        self.name = name
        self.module = module

    def __str__(self):
        return "Rc::new({}::{})".format(self.module, self.name)


class Opcode:
    SEG_DS = "DS"
    SEG_ES = "ES"
    SEG_CS = "CS"

    NUM_ARGS_ZERO = "NumArgs::Zero"
    NUM_ARGS_ONE = "NumArgs::One"
    NUM_ARGS_TWO = "NumArgs::Two"

    FLAG_IMMEDIATE = "Immediate"
    FLAG_SIZE_MISMATCH = "SizeMismatch"
    FLAG_NOP = "Nop"
    FLAG_FORCE_WORD = "ForceWord"
    FLAG_FORCE_BYTE = "ForceByte"
    FLAG_FORCE_DWORD = "ForceDWord"
    FLAG_FORCE_DIRECTION = "ForceDirection"

    def __init__(self, num_args, action, mnemonic, shorthand1=None, shorthand2=None, flags=(), segment=SEG_DS):
        self.num_args = num_args
        self.action = action
        self.mnemonic = mnemonic
        self.shorthand1 = shorthand1
        self.shorthand2 = shorthand2
        self.flags = flags
        self.segment = segment

    @staticmethod
    def get_default():
        return Opcode(0, Function("undefined", ""), "UD")

    def get_mnemonic(self):
        if type(self.mnemonic) == str:
            return "Mnemonic::Static(String::from(\"{}\"))".format(self.mnemonic)
        return "Mnemonic::Dynamic({!s})".format(self.mnemonic)

    def get_shorthand1(self):
        if self.shorthand1 is None:
            return "None"
        else:
            return "Some(Placeholder::{})".format(self.shorthand1)

    def get_shorthand2(self):
        if self.shorthand2 is None:
            return "None"
        else:
            return "Some(Placeholder::{})".format(self.shorthand2)

    def get_flags(self):
        flags = "make_bitflags!(OpcodeFlags::{ "

        for i in range(len(self.flags) - 1):
            flags += "{} | ".format(self.flags[i])
        if len(self.flags) > 0:
            flags += "{}".format(self.flags[len(self.flags)-1])

        flags += " })"
        return flags

    def __str__(self):
        return \
            "Some(Opcode{{ num_args: {!s}, action: {!s}, mnemonic: {!s}, shorthand1: {!s}," \
            " shorthand2: {!s}, flags: {!s}, segment: Regs::{!s} }})" \
            .format(self.num_args, self.action, self.get_mnemonic(), self.get_shorthand1(), self.get_shorthand2(),
                    self.get_flags(), self.segment)


def condition_to_opcode(func, mnemonic, prefix):
    param = 'this' if 'this' in func else '_'
    return 'Box::new(|{}: &CPU| {})'.format(param, func), prefix + mnemonic


def make_flag_opcodes():
    flag_opcodes = [
        ('this.check_flag(CPUFlags::OVERFLOW)', 'o'),
        ('!this.check_flag(CPUFlags::OVERFLOW)', 'no'),
        ('this.check_flag(CPUFlags::CARRY)', 'c'),
        ('!this.check_flag(CPUFlags::CARRY)', 'nc'),
        ('this.check_flag(CPUFlags::ZERO)', 'e'),
        ('!this.check_flag(CPUFlags::ZERO)', 'ne'),
        ('this.check_flag(CPUFlags::CARRY) || this.check_flag(CPUFlags::ZERO)', 'be'),
        ('!this.check_flag(CPUFlags::CARRY) && !this.check_flag(CPUFlags::ZERO)', 'a'),
        ('this.check_flag(CPUFlags::SIGN)', 's'),
        ('!this.check_flag(CPUFlags::SIGN)', 'ns'),
        ('this.check_flag(CPUFlags::PARITY)', 'p'),
        ('!this.check_flag(CPUFlags::PARITY)', 'np'),
        ('this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW)', 'l'),
        ('!this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW)', 'ge'),
        ('this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW) || this.check_flag(CPUFlags::ZERO)', 'le'),
        ('this.check_flag(CPUFlags::SIGN) && !this.check_flags_not_equal(CPUFlags::SIGN, CPUFlags::OVERFLOW)', 'g')
    ]
    return [condition_to_opcode(func, mnemonic, 'j') for (func, mnemonic) in flag_opcodes]


def make_loop_opcodes():
    loop_opcodes = [
        ('!this.check_flag(CPUFlags::ZERO)', 'ne'),
        ('this.check_flag(CPUFlags::ZERO)', 'e'),
        ('true', '')
    ]
    return [condition_to_opcode(func, mnemonic, 'loop') for (func, mnemonic) in loop_opcodes]


def make_opcodes():
    opcodes = {
        0x90: Opcode(Opcode.NUM_ARGS_ZERO, Function('nop', 'mem'), 'nop', flags=(Opcode.FLAG_NOP,)),
        0x88: Opcode(Opcode.NUM_ARGS_TWO, Function('mov', 'mem'), 'mov'),
        0xA0: Opcode(Opcode.NUM_ARGS_TWO, Function('mov', 'mem'), 'mov', 'Reg(0)', 'Ptr'),
        0xC6: Opcode(Opcode.NUM_ARGS_TWO, Function('mov', 'mem'), 'mov', flags=(Opcode.FLAG_IMMEDIATE,)),
        0x86: Opcode(Opcode.NUM_ARGS_TWO, Function('xchg', 'mem'), 'xchg'),
        0xD7: Opcode(Opcode.NUM_ARGS_ZERO, Function('xlat', 'mem'), 'xlat'),
        0xC5: Opcode(Opcode.NUM_ARGS_TWO, Function('lds', 'mem'), 'lds', flags=(Opcode.FLAG_FORCE_DWORD,)),
        0xC4: Opcode(Opcode.NUM_ARGS_TWO, Function('les', 'mem'), 'les', flags=(Opcode.FLAG_FORCE_DWORD,)),
        0x8D: Opcode(Opcode.NUM_ARGS_TWO, Function('lea', 'mem'), 'lea', flags=(Opcode.FLAG_FORCE_DIRECTION,)),
        0xA4: Opcode(Opcode.NUM_ARGS_ZERO, Function('movs', 'mem'), 'movsb', shorthand1='Byte(0)'),
        0xA5: Opcode(Opcode.NUM_ARGS_ZERO, Function('movs', 'mem'), 'movsw', shorthand1='Word(0)'),
        0xAC: Opcode(Opcode.NUM_ARGS_ZERO, Function('lods', 'mem'), 'lodsb', shorthand1='Byte(0)'),
        0xAD: Opcode(Opcode.NUM_ARGS_ZERO, Function('lods', 'mem'), 'lodsw', shorthand1='Word(0)'),
        0xAA: Opcode(Opcode.NUM_ARGS_ZERO, Function('stos', 'mem'), 'stosb', shorthand1='Byte(0)',
                     segment=Opcode.SEG_ES),
        0xAB: Opcode(Opcode.NUM_ARGS_ZERO, Function('stos', 'mem'), 'stosw', shorthand1='Word(0)',
                     segment=Opcode.SEG_ES),
        0x98: Opcode(Opcode.NUM_ARGS_ZERO, Function('cbw', 'mem'), 'cbw'),
        0x99: Opcode(Opcode.NUM_ARGS_ZERO, Function('cwd', 'mem'), 'cwd'),
        0x80: Opcode(Opcode.NUM_ARGS_TWO, Function('alu_dispatch_two_args', 'alu'),
                     Function('alu_dispatch_two_args_mnemonic', 'alu'), flags=(Opcode.FLAG_IMMEDIATE,)),
        0x83: Opcode(Opcode.NUM_ARGS_TWO, Function('alu_dispatch_two_args', 'alu'),
                     Function('alu_dispatch_two_args_mnemonic', 'alu'),
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_SIZE_MISMATCH)),
        0xFE: Opcode(Opcode.NUM_ARGS_ONE, Function('alu_dispatch_one_arg', 'alu'),
                     Function('alu_dispatch_one_arg_mnemonic', 'alu')),
        0xF6: Opcode(Opcode.NUM_ARGS_ONE, Function('mul_dispatch', 'alu'), Function('mul_dispatch_mnemonic', 'alu')),
        0xC0: Opcode(Opcode.NUM_ARGS_TWO, Function('rotate_dispatch', 'alu'),
                     Function('rotate_dispatch_mnemonic', 'alu'), shorthand2='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_BYTE, Opcode.FLAG_SIZE_MISMATCH)),
        0xD0: Opcode(Opcode.NUM_ARGS_TWO, Function('rotate_dispatch', 'alu'),
                     Function('rotate_dispatch_mnemonic', 'alu'), shorthand2='Byte(1)',
                     flags=(Opcode.FLAG_SIZE_MISMATCH,)),
        0xD3: Opcode(Opcode.NUM_ARGS_TWO, Function('rotate_dispatch', 'alu'),
                     Function('rotate_dispatch_mnemonic', 'alu'), shorthand2='Reg8(2)',
                     flags=(Opcode.FLAG_SIZE_MISMATCH,)),
        0x18: Opcode(Opcode.NUM_ARGS_TWO, Function('sbb', 'alu'), 'sbb'),
        0x1C: Opcode(Opcode.NUM_ARGS_TWO, Function('sbb', 'alu'), 'sbb', shorthand1='Reg(0)', shorthand2='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE,)),
        0x37: Opcode(Opcode.NUM_ARGS_ZERO, Function('aaa', 'alu'), 'aaa'),
        0xD5: Opcode(Opcode.NUM_ARGS_ONE, Function('aad', 'alu'), 'aad',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_BYTE)),
        0x3F: Opcode(Opcode.NUM_ARGS_ZERO, Function('aas', 'alu'), 'aas'),
        0x27: Opcode(Opcode.NUM_ARGS_ZERO, Function('daa', 'alu'), 'daa'),
        0x14: Opcode(Opcode.NUM_ARGS_TWO, Function('adc', 'alu'), 'adc', shorthand1='Reg8(0)', shorthand2='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE,)),
        0x10: Opcode(Opcode.NUM_ARGS_TWO, Function('adc', 'alu'), 'adc'),
        0x68: Opcode(Opcode.NUM_ARGS_ONE, Function('push', 'stack'), 'push', shorthand1='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_WORD)),
        0x6A: Opcode(Opcode.NUM_ARGS_ONE,  Function('push', 'stack'), 'push', shorthand1='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_BYTE)),
        0x06: Opcode(Opcode.NUM_ARGS_ONE, Function('push', 'stack'), 'push', shorthand1='RegEnum(Regs::ES)'),
        0x0E: Opcode(Opcode.NUM_ARGS_ONE, Function('push', 'stack'), 'push', shorthand1='RegEnum(Regs::CS)'),
        0x16: Opcode(Opcode.NUM_ARGS_ONE, Function('push', 'stack'), 'push', shorthand1='RegEnum(Regs::SS)'),
        0x1E: Opcode(Opcode.NUM_ARGS_ONE, Function('push', 'stack'), 'push', shorthand1='RegEnum(Regs::DS)'),
        0x8F: Opcode(Opcode.NUM_ARGS_ONE, Function('pop', 'stack'), 'pop'),
        0x1F: Opcode(Opcode.NUM_ARGS_ONE, Function('pop', 'stack'), 'pop', shorthand1='RegEnum(Regs::DS)'),
        0x07: Opcode(Opcode.NUM_ARGS_ONE, Function('pop', 'stack'), 'pop', shorthand1='RegEnum(Regs::ES)'),
        0x17: Opcode(Opcode.NUM_ARGS_ONE, Function('pop', 'stack'), 'pop', shorthand1='RegEnum(Regs::SS)'),
        0x9C: Opcode(Opcode.NUM_ARGS_ZERO, Function('push', 'stack'), 'pushf', shorthand1='RegEnum(Regs::FLAGS)'),
        0x9D: Opcode(Opcode.NUM_ARGS_ZERO, Function('pop', 'stack'), 'popf', shorthand1='RegEnum(Regs::FLAGS)'),
        0x60: Opcode(Opcode.NUM_ARGS_ZERO, Function('pusha', 'stack'), 'pusha'),
        0x61: Opcode(Opcode.NUM_ARGS_ZERO, Function('popa', 'stack'), 'popa'),
        0xE8: Opcode(Opcode.NUM_ARGS_ONE, Function('near_call', 'stack'), 'call',
                     flags=(Opcode.FLAG_FORCE_WORD, Opcode.FLAG_IMMEDIATE)),
        0x9A: Opcode(Opcode.NUM_ARGS_ONE, Function('far_call', 'stack'), 'call',
                     flags=(Opcode.FLAG_FORCE_WORD, Opcode.FLAG_IMMEDIATE)),
        0xCB: Opcode(Opcode.NUM_ARGS_ZERO, Function('far_ret', 'stack'), 'ret'),
        0xC3: Opcode(Opcode.NUM_ARGS_ZERO, Function('near_ret', 'stack'), 'ret'),
        0xCA: Opcode(Opcode.NUM_ARGS_ONE, Function('far_ret', 'stack'), 'ret', shorthand1='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_WORD)),
        0xC2: Opcode(Opcode.NUM_ARGS_ONE, Function('near_ret', 'stack'), 'ret', shorthand1='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_WORD)),
        0xC8: Opcode(Opcode.NUM_ARGS_TWO, Function('enter', 'stack'), 'enter', shorthand1='Imm', shorthand2='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_SIZE_MISMATCH)),
        0xC9: Opcode(Opcode.NUM_ARGS_ZERO, Function('leave', 'stack'), 'leave'),
        0xE9: Opcode(Opcode.NUM_ARGS_ONE, Function('jmp', 'jmp'), 'jmp', segment=Opcode.SEG_CS,
                     flags=(Opcode.FLAG_IMMEDIATE,)),
        0xE3: Opcode(Opcode.NUM_ARGS_ONE, 'jmp::cond_jmp({})'
                     .format(condition_to_opcode('this.regs.get(&Regs::CX).unwrap().value == 0', 'cxz', 'j')[0]),
                     'jcxz', segment=Opcode.SEG_CS, flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_SIZE_MISMATCH)),
        0xF8: Opcode(Opcode.NUM_ARGS_ZERO, Function('clc', 'flags'), 'clc'),
        0xFC: Opcode(Opcode.NUM_ARGS_ZERO, Function('cld', 'flags'), 'cld'),
        0xFA: Opcode(Opcode.NUM_ARGS_ZERO, Function('cli', 'flags'), 'cli'),
        0xF5: Opcode(Opcode.NUM_ARGS_ZERO, Function('cmc', 'flags'), 'cmc'),
        0xF9: Opcode(Opcode.NUM_ARGS_ZERO, Function('stc', 'flags'), 'stc'),
        0xFD: Opcode(Opcode.NUM_ARGS_ZERO, Function('std', 'flags'), 'std'),
        0xFB: Opcode(Opcode.NUM_ARGS_ZERO, Function('sti', 'flags'), 'sti'),
        0x38: Opcode(Opcode.NUM_ARGS_TWO, Function('cmp', 'flags'), 'cmp', flags=(Opcode.FLAG_SIZE_MISMATCH,)),
        0x3C: Opcode(Opcode.NUM_ARGS_TWO, Function('cmp', 'flags'), 'cmp', shorthand1='Reg(0)', shorthand2='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE,)),
        0x84: Opcode(Opcode.NUM_ARGS_TWO, Function('test', 'flags'), 'test'),
        0xA8: Opcode(Opcode.NUM_ARGS_TWO, Function('test', 'flags'), 'test', shorthand1='Reg(0)', shorthand2='Imm',
                     flags=(Opcode.FLAG_IMMEDIATE,)),
        0xA6: Opcode(Opcode.NUM_ARGS_ZERO, Function('cmps', 'flags'), 'cmpsb',
                     shorthand1='Byte(0)'),
        0xA7: Opcode(Opcode.NUM_ARGS_ZERO, Function('cmps', 'flags'), 'cmpsw',
                     shorthand1='Word(0)'),
        0xAE: Opcode(Opcode.NUM_ARGS_ZERO, Function('scas', 'flags'), 'scasb', shorthand1='Reg8(0)',
                     segment=Opcode.SEG_ES),
        0xAF: Opcode(Opcode.NUM_ARGS_ZERO, Function('scas', 'flags'), 'scasw', shorthand1='Reg16(0)',
                     segment=Opcode.SEG_ES),
        0x9F: Opcode(Opcode.NUM_ARGS_ZERO, Function('lahf', 'flags'), 'lahf'),
        0x9E: Opcode(Opcode.NUM_ARGS_ZERO, Function('sahf', 'flags'), 'sahf'),
        0xF3: Opcode(Opcode.NUM_ARGS_ONE, Function('rep', 'flags'), Function('rep_mnemonic', 'flags'),
                     shorthand1='Opcode'),
        0xF2: Opcode(Opcode.NUM_ARGS_ONE, Function('repne', 'flags'), 'repne', shorthand1='Opcode'),
        0xCD: Opcode(Opcode.NUM_ARGS_ONE, Function('int_req', 'int'), 'int',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_BYTE)),
        0xCC: Opcode(Opcode.NUM_ARGS_ONE, Function('int_req', 'int'), 'int',
                     flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_FORCE_BYTE), shorthand1='Byte(3)'),
        0xCE: Opcode(Opcode.NUM_ARGS_ZERO, Function('into', 'int'), 'into'),
        0xCF: Opcode(Opcode.NUM_ARGS_ZERO, Function('iret', 'int'), 'int'),
        0x62: Opcode(Opcode.NUM_ARGS_TWO, Function('bound', 'int'), 'bound', flags=(Opcode.FLAG_FORCE_DWORD,))
    }

    for i in range(8):
        opcodes[0xB0 + i] = Opcode(Opcode.NUM_ARGS_TWO, Function('mov', 'mem'), 'mov', 'Reg8({})'.format(i), 'Imm',
                                   (Opcode.FLAG_IMMEDIATE,))
        opcodes[0xB8 + i] = Opcode(Opcode.NUM_ARGS_TWO, Function('mov', 'mem'), 'mov', 'Reg16({})'.format(i), 'Imm',
                                   (Opcode.FLAG_IMMEDIATE,))
        if i > 0:
            opcodes[0x90 + i] = Opcode(Opcode.NUM_ARGS_TWO, Function('xchg', 'mem'), 'xchg', shorthand1='Reg16(0)',
                                       shorthand2='Reg16({})'.format(i))
        opcodes[0x40 + i] = Opcode(Opcode.NUM_ARGS_ONE, Function('inc', 'alu'), 'inc', shorthand1='Reg16({})'.format(i))
        opcodes[0x48 + i] = Opcode(Opcode.NUM_ARGS_ONE, Function('dec', 'alu'), 'dec', shorthand1='Reg16({})'.format(i))
        opcodes[0x50 + i] = Opcode(Opcode.NUM_ARGS_ONE, Function('push', 'stack'), 'push',
                                   shorthand1='Reg16({})'.format(i))
        opcodes[0x58 + i] = Opcode(Opcode.NUM_ARGS_ONE, Function('pop', 'stack'), 'pop',
                                   shorthand1='Reg16({})'.format(i))

    alu_opcodes = {
        0x00: (Function('add', 'alu'), 'add'),
        0x28: (Function('sub', 'alu'), 'sub'),
        0x30: (Function('xor', 'alu'), 'xor'),
        0x20: (Function('and', 'alu'), 'and'),
        0x08: (Function('or', 'alu'), 'or')
    }

    for (offset, (action, mnemonic)) in alu_opcodes.items():
        opcodes[0x00 + offset] = Opcode(Opcode.NUM_ARGS_TWO, action, mnemonic)
        opcodes[0x04 + offset] = Opcode(Opcode.NUM_ARGS_TWO, action, mnemonic, shorthand1='Reg(0)', shorthand2='Imm',
                                        flags=(Opcode.FLAG_IMMEDIATE,))

    for i, (action, mnemonic) in enumerate(make_flag_opcodes()):
        opcodes[0x70 + i] = Opcode(Opcode.NUM_ARGS_ONE, 'jmp::cond_jmp({})'.format(action), mnemonic,
                                   segment=Opcode.SEG_CS, flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_SIZE_MISMATCH))

    for i, (action, mnemonic) in enumerate(make_loop_opcodes()):
        opcodes[0xE0 + i] = Opcode(Opcode.NUM_ARGS_ONE, 'jmp::lop({})'.format(action), mnemonic,
                                   segment=Opcode.SEG_CS, flags=(Opcode.FLAG_IMMEDIATE, Opcode.FLAG_SIZE_MISMATCH))

    opcodes_array = []
    for i in range(256):
        if i in opcodes:
            opcodes_array.append(str(opcodes[i]))
        else:
            opcodes_array.append("None")

    return opcodes_array


def dump_opcodes(opcodes, file):
    with file.open('w') as f:
        f.write("/// This file was generated by the Python script in utils/opcode_generator\n\n"
                "use crate::cpu::instruction::opcode::{Opcode, Mnemonic, NumArgs, Placeholder};\n"
                "use crate::cpu::{Regs, CPU, CPUFlags};\n"
                "use crate::cpu::instruction::actions::{alu, flags, int, jmp, mem, stack};\n"
                "use enumflags2::make_bitflags;\n"
                "use crate::cpu::instruction::opcode::OpcodeFlags;\n"
                "use std::rc::Rc;\n\n"
                "impl Opcode {\n"
                "\tpub fn get_opcode_data() -> [Option<Opcode>; 256] {\n\t\t[\n")
        f.writelines(['\t\t\t' + str(opcode) + ',\n' for opcode in opcodes])
        f.write("\t\t]\n\t}\n}\n\n")


def main():
    dump_opcodes(make_opcodes(), pathlib.Path('.') / 'xtreme86' / 'src' / 'cpu' / 'instruction' / 'data.rs')


if __name__ == '__main__':
    main()
