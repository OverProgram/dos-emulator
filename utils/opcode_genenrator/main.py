import pathlib


class Function:
    def __init__(self, name, module):
        self.name = name
        self.module = module

    def __str__(self):
        return "Arc::new({}::{})".format(self.module, self.name)


class Opcode:
    SEG_DS = "DS"

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
        elif type(self.mnemonic) == Function:
            return "Mnemonic::Dynamic({!s})".format(self.mnemonic)

    def get_shorthand1(self):
        if self.shorthand1 is None:
            return "None"
        else:
            return "Some({})".format(self.shorthand1)

    def get_shorthand2(self):
        if self.shorthand2 is None:
            return "None"
        else:
            return "Some({})".format(self.shorthand2)

    def get_flags(self):
        flags = "make_bitflags!(OpcodeFlags::{ "

        for i in range(len(self.flags) - 1):
            flags += "{} | ".format(self.flags[i])
        if len(self.flags) > 1:
            flags += "{} ".format(self.flags[len(self.flags)])

        flags += " })"
        return flags

    def __str__(self):
        return \
            "Some(Opcode{{ num_args: {!s}, action: {!s}, mnemonic: {!s}, shorthand1: {!s}," \
            " shorthand2: {!s}, flags: {!s}, segment: Regs::{!s} }})" \
            .format(self.num_args, self.action, self.get_mnemonic(), self.get_shorthand1(), self.get_shorthand2(),
                    self.get_flags(), self.segment)


def make_opcodes():
    opcodes = {}

    opcodes[0x01] = Opcode(Opcode.NUM_ARGS_TWO, Function("mov", "mem"), "mov")

    opcodes_array = []
    for i in range(256):
        if i in opcodes:
            opcodes_array.append(str(opcodes[i]))
        else:
            opcodes_array.append("None")

    return opcodes_array


def dump_opcodes(opcodes, file):
    with file.open('w') as f:
        f.write("use crate::cpu::instruction::opcode::{Opcode, Mnemonic, NumArgs};\n"
                "use crate::cpu::{Regs};\n"
                "use crate::cpu::instruction::actions::{alu, flags, int, jmp, mem, stack};\n"
                "use enumflags2::make_bitflags;\n"
                "use crate::cpu::instruction::opcode::OpcodeFlags;\n"
                "use std::sync::Arc;\n\n"
                "lazy_static! {\n"
                "\tpub static ref OPCODE_DATA: [Option<Opcode>; 256] = [\n")
        f.writelines(['\t\t' + str(opcode) + ',\n' for opcode in opcodes])
        f.write("\t];\n}\n\n")


def main():
    dump_opcodes(make_opcodes(), pathlib.Path('.') / 'xtreme86' / 'src' / 'cpu' / 'instruction' / 'data.rs')


if __name__ == '__main__':
    main()
