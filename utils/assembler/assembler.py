import subprocess
import sys
import tempfile


def call_cmd(cmd):
    cmd = cmd.split()
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return process.communicate()


lines = []
should_continue = True


def generate_code():
    code = ".code16\n.intel_syntax noprefix\n.text\n"
    for line in lines:
        code += "{}\n".format(line)
    return code


def compile_code(code):
    asm_file = tempfile.NamedTemporaryFile(mode='w+')
    obj_file = tempfile.NamedTemporaryFile()
    bin_file = tempfile.NamedTemporaryFile(mode='rb')
    asm_file.write(code)
    asm_file.flush()
    stdout, stderr = call_cmd("as {} -o {}".format(asm_file.name, obj_file.name))
    if stderr:
        print("error: {}".format(stderr.decode('utf-8')), file=sys.stderr)
        asm_file.close()
        tmp = open(obj_file.name, 'w')
        tmp.close()
        return
    call_cmd("objcopy -O binary {} {}".format(obj_file.name, bin_file.name))
    hexdump = bin_file.read()
    asm_file.close()
    obj_file.close()
    bin_file.close()
    return hexdump


def format_dump(dump_bytes):
    format_dump_str = ""
    for i in range(len(dump_bytes)):
        byte = dump_bytes[i:i+1]
        format_dump_str += '{}, '.format(hex(int.from_bytes(byte, 'little')))
    return format_dump_str[:len(format_dump_str)-2]


def dump():
    global lines
    code = compile_code(generate_code())
    if code is not None:
        print(format_dump(code))
    lines = []


def load(filename):
    try:
        f = open(filename, 'r')
        lines.extend(f)
    except FileNotFoundError:
        print('Error: {} does not exist'.format(filename))


def save(filename):
    global lines
    try:
        f = open(filename, 'wb')
        code = compile_code(generate_code())
        if code is not None:
            f.write(code)
    except FileNotFoundError:
        print('Error: {} does not exist'.format(filename))
    finally:
        lines = []


def stop_loop():
    global should_continue
    should_continue = False


keywords = {
    "dump": dump,
    "save": save,
    "load": load,
    "quit": stop_loop
}


def main():
    i = 0
    while should_continue:
        line = input("In[{}]: ".format(i))
        if line.split(' ')[0] in keywords.keys():
            keywords[line.split(' ')[0]](*line.split(' ')[1:])
        else:
            lines.append(line)
        i += 1


if __name__ == '__main__':
    main()
