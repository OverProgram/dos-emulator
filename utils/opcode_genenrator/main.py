import pathlib


class Function:
    def __init__(self, name, module):
        self.name = name
        self.module = module

    def __str__(self):
        return "{}::{}".format(self.name, self.module)


class Opcode:
    SEG_DS = "DS"

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
        return str(self.mnemonic)

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
            "Opcode{ num_args: {}, action: {}, mnemonic: {}, shorthand1: {}, shorthand2: {}, flags: {}, segment: {} }" \
            .format(self.num_args, self.action, self.get_mnemonic(), self.get_shorthand1(), self.get_shorthand2(),
                    self.get_flags(), self.segment)


def make_opcodes():
    opcodes = {}

    opcodes_array = []
    for i in range(256):
        if i in opcodes:
            opcodes_array.append(str(opcodes[i]))
        else:
            opcodes_array.append("Opcode")

    return


def dump_opcodes(opcodes, file):
    with file.open('w+') as f:
        f.write("use crate::cpu::instruction::opcode::Opcode\n\nconst OPCODE_DATA: [Opcodes; 256] = [\n")
        f.writelines([str(opcode) for opcode in opcodes])
        f.write("];\n\n")


def main():
    dump_opcodes(make_opcodes(), pathlib.Path('.') / 'xtreme86' / 'src' / 'cpu' / 'instruction' / 'data.rs')


if __name__ == '__main__':
    main()
