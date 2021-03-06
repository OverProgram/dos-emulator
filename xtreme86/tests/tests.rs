use xtreme86::cpu;
use std::fs::File;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

fn load_binary(filename: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("assets");
    path.push(filename);

    let mut f = File::open(&path).expect("No file found!");
    let metadata = fs::metadata(&path).expect("No file found!");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("Couldn't read file");
    buffer
}

fn new_cpu_from_file(filename: &str) -> cpu::CPU {
    let mut computer = cpu::CPU::new(0x7FFFFF);
    computer.set_reg(cpu::Regs::SS, 0x003F);
    computer.set_reg(cpu::Regs::CS, 0x103F);
    computer.set_reg(cpu::Regs::DS, 0x203F);
    computer.set_reg(cpu::Regs::SP, 0xFFFF);
    computer.set_reg(cpu::Regs::IP, 0x0000);

    let buffer = load_binary(filename);
    computer.load(buffer, 0x103F0);

    computer
}

fn new_cpu_vec(code: Vec<u8>) -> cpu::CPU {
    let mut computer = cpu::CPU::new(code.len());
    computer.load(code, 0);
    computer
}

mod mov_test {
    use super::cpu;
    use crate::new_cpu_from_file;
    use crate::cpu::Regs;
    use xtreme86::cpu::CPU;

    #[test]
    fn test_mov_reg_shorthand() {
        let code = vec![0xB8, 0x06, 0x00];    // mov ax, 0x6
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        assert_eq!(computer.read_reg(cpu::Regs::AX).unwrap(), 6);
    }

    #[test]
    fn test_mov_reg() {
        let code = vec![0x66, 0xc6, 0x6, 0x0, 0x0, 0x0];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.probe_mem(0), 0x0);
    }

    #[test]
    fn test_mov_ptr() {
        let code = vec![0x00, 0xc6, 0x06, 0x00, 0x00, 0x55];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.probe_mem(0), 0x55);
    }

    #[test]
    fn test_seg_override() {
        let mut comp = new_cpu_from_file("obj/seg.out");

        comp.run_to_nop(0);
        assert_eq!(comp.probe_mem_es(0), 0x05);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.probe_mem_word(CPU::physical_address(comp.read_reg(Regs::SS).unwrap(), 0x06) as usize), 0xFFFF);
    }

    #[test]
    fn test_lea_convert() {
        let mut comp = new_cpu_from_file("obj/lea.out");
        comp.run_to_nop(0);
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 9);
        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0xFFFF);
        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::DX).unwrap(), 0xFFFF);
    }

    #[test]
    fn test_load() {
        let mut comp = new_cpu_from_file("obj/load.out");
        comp.write_bytes_ds(0, vec![0x00, 0x00, 0xFF, 0xFF]).unwrap();
        comp.run_to_nop(0);
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x0000);
        assert_eq!(comp.read_reg(Regs::ES).unwrap(), 0xFFFF);
        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x0000);
        assert_eq!(comp.read_reg(Regs::DS).unwrap(), 0xFFFF);
    }

    #[test]
    fn test_ex() {
        let mut comp = new_cpu_from_file("obj/ex.out");
        comp.write_bytes_ds(0x0A, vec![0xFF]).unwrap();
        comp.run_to_nop(0);
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x20);
        assert_eq!(comp.read_reg(Regs::DX).unwrap(), 0x10);
        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0xFF);
    }
}

mod test_alu {
    use super::cpu;
    use crate::cpu::Regs;
    use crate::{new_cpu_vec, new_cpu_from_file};

    #[test]
    fn test_add() {
        let code = vec![0x83, 0xc0, 0x5];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        assert_eq!(computer.read_reg(cpu::Regs::AX).unwrap(), 5);
    }

    #[test]
    fn test_sub() {
        let code = vec![0x30, 0x80, 0x2e, 0x0, 0x0, 0x20];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.probe_mem(0), 0x10);
    }

    #[test]
    fn test_and() {
        let code = vec![0xb8, 0xFF, 0x0, 0xbb, 0xaa, 0x0, 0x21, 0xc3];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        computer.execute_next();
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::BX).unwrap(), 0xAA);
    }

    #[test]
    fn test_or() {
        let code = vec![0xaa, 0x00, 0xb9, 0x55, 0x0, 0xb, 0xe, 0x0, 0x0];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(2);
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::CX).unwrap(), 0xFF);
    }

    #[test]
    fn test_inc_reg() {
        let code = vec![0x40];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x01);
    }

    #[test]
    fn test_dec_mem() {
        let code = vec![0xff, 0xfe, 0xe, 0x0, 0x0];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.probe_mem(0), 0xFE)
    }

    #[test]
    fn test_flags() {
        let mut computer = new_cpu_vec(vec![0xff, 0xfe, 0x6, 0x0, 0x0]);
        computer.execute_next_from(1);
        assert_eq!(computer.read_reg(Regs::FLAGS).unwrap() & (cpu::CPUFlags::OVERFLOW | cpu::CPUFlags::ZERO), (cpu::CPUFlags::OVERFLOW | cpu::CPUFlags::ZERO));
    }

    #[test]
    fn test_mul() {
        let mut computer = new_cpu_vec(vec![0xb8, 0x55, 0x0, 0xbb, 0xaa, 0x0, 0xf7, 0xe3]);
        computer.execute_next();
        computer.execute_next();
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x3872);
        assert_eq!(computer.read_reg(Regs::DX).unwrap(), 0x00);
    }

    #[test]
    fn test_div() {
        let mut computer = new_cpu_vec(vec![0xba, 0xaa, 0x00, 0xb8, 0x55, 0x55, 0xbb, 0xff, 0x00, 0xf7, 0xfb]);
        computer.execute_next();
        computer.execute_next();
        computer.execute_next();
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0xab00);
        assert_eq!(computer.read_reg(Regs::DX).unwrap(), 0x0055);
    }

    #[test]
    fn test_misc() {
        let mut computer = new_cpu_from_file("obj/alu.out");
        computer.run_to_nop(0);
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x0000);
        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap() & 0xFF, 9);
        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::BX).unwrap() & 0xFF, 9);
        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x02);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 11);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0xFF00);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x00FF);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x0001);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x0072);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x0005);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x0014);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x1508);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x0088);
    }

    #[test]
    fn test_shift() {
        let mut comp = new_cpu_from_file("obj/shift.out");

        comp.run_to_nop(0);
        assert_eq!(comp.read_reg(Regs::CX).unwrap(), 0x7FFF);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::CX).unwrap(), 0xFFFE);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::DX).unwrap(), 0xFF00);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::DX).unwrap(), 0xFE00);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::BX).unwrap(), 0x00F0);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::BX).unwrap(), 0x000F);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::SI).unwrap(), 0x8003);
    }
}

mod stack_test {
    use crate::cpu::Regs;
    use crate::{new_cpu_from_file};

    #[test]
    fn test_push() {
        let mut computer = new_cpu_from_file("obj/push.out");
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x05);
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::SP).unwrap(), 0xFFFD);
        assert_eq!(computer.get_mem_seg(Regs::SS, computer.read_reg(Regs::SP).unwrap() + 1), 0x05);
    }

    #[test]
    fn test_pop() {
        let mut computer = new_cpu_from_file("obj/pop.out");
        computer.execute_next();
        computer.execute_next();
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::BX).unwrap(), 0x05);
    }

    #[test]
    fn test_proc() {
        let mut computer = new_cpu_from_file("obj/proc.out");
        computer.run_to_nop(0);
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x16);
        assert_eq!(computer.read_reg(Regs::SP).unwrap(), 0xFFFF);

        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::SP).unwrap(), 0xFFFF);
    }
}

mod jmp_test {
    use crate::cpu::Regs;
    use crate::{new_cpu_from_file, load_binary};
    use xtreme86::cpu::CPU;

    #[test]
    fn test_jmp() {
        let mut computer = new_cpu_from_file("obj/jmp.out");
        computer.execute_next();
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x06);
    }

    #[test]
    fn test_cond_jmp() {
        let mut computer = new_cpu_from_file("obj/jmp_cond.out");
        computer.run_to_nop(0);
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x16);
    }

    fn fib(n: usize) -> u16 {
        let mut f: Vec<u16>= Vec::with_capacity(n + 2);
        f.push(0);
        f.push(1);
        for i in 2..=n {
            f.push(f[i - 1] + f[i - 2]);
        }

        f[n]
    }

    #[test]
    fn test_loop() {
        let mut comp = new_cpu_from_file("obj/fib.out");
        comp.run_to_nop(0);
        for i in 0..10 as usize {
            if i > 1 {
                comp.run_to_nop_from_ip();
            }
            let address = comp.address_in_ds((i * 2) as u16) as usize;
            if comp.probe_mem_word(address) != fib(i) {
                println!("{}", i);
                assert_eq!(comp.probe_mem_word(address), fib(i));
            }
        }
    }

    #[test]
    fn test_far() {
        let mut comp = new_cpu_from_file("obj/far_code.out");
        let code = load_binary("obj/far.out");

        comp.load(code, CPU::physical_address(0x8000, 0) as usize);

        comp.run_to_nop(0);
        assert_eq!(comp.read_reg(Regs::SP).unwrap(), 0xFFFF);
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x05);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x10);
    }
}

mod int_test {
    use crate::new_cpu_from_file;
    use crate::cpu::Regs;

    #[test]
    fn test_soft_int() {
        let mut computer = new_cpu_from_file("obj/int.out");
        computer.load(vec![0x14, 0x00, 0x3F, 0x10], 0);
        computer.load(vec![0x14, 0x00, 0x3F, 0x10], 20);
        computer.load(vec![0x14, 0x00, 0x3F, 0x10], 16);
        computer.write_bytes_ds(0, vec![0x00, 0x00, 0x05, 0x00]).unwrap();
        computer.run_to_nop(0);
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 5);
        assert_eq!(computer.read_reg(Regs::SP).unwrap(), 0xFFFF);
        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 5);
        assert_eq!(computer.read_reg(Regs::SP).unwrap(), 0xFFFF);
        computer.run_to_nop_from_ip();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 5);
        assert_eq!(computer.read_reg(Regs::SP).unwrap(), 0xFFFF);
    }
}

mod test_flags {
    use crate::new_cpu_from_file;
    use crate::cpu::Regs;

    #[test]
    fn test_cmp() {
        let mut comp = new_cpu_from_file("obj/cmp.out");
        comp.run_to_nop(0);
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x30);
        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x30);
    }

    #[test]
    fn test_flag() {
        let mut comp = new_cpu_from_file("obj/flag.out");
        comp.run_to_nop(0);
        assert_ne!(comp.read_reg(Regs::FLAGS).unwrap() & 0x80, 0);
    }
}

mod test_string {
    use crate::new_cpu_from_file;
    use xtreme86::cpu::{Regs, CPU};

    #[test]
    fn test_string() {
        let mut comp = new_cpu_from_file("obj/str.out");
        let str1 = "Hello, world!".to_string().into_bytes();
        let str2 = "Hello, aayal!".to_string().into_bytes();

        comp.write_bytes_ds(0, str1).unwrap();
        comp.write_bytes_es(0, str2).unwrap();

        comp.run_to_nop(0);
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 20);
        assert_eq!(comp.read_reg(Regs::SI).unwrap(), 8);
        assert_eq!(comp.read_reg(Regs::DI).unwrap(), 8);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::DI).unwrap(), 0);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::AX).unwrap(), 0x726F);

        comp.run_to_nop_from_ip();
        let address_ds = CPU::physical_address(comp.read_reg(Regs::DS).unwrap(), 7);
        let address_es = CPU::physical_address(comp.read_reg(Regs::ES).unwrap(), 7);

        let val_ds = comp.probe_mem(address_ds as usize);

        assert_eq!(comp.probe_mem(address_es as usize), val_ds);

        comp.run_to_nop_from_ip();
        let address_es_new = address_es + 1;
        assert_eq!(comp.probe_mem_word(address_es_new as usize), 0x706F);

        comp.run_to_nop_from_ip();
        assert_eq!(comp.read_reg(Regs::SI).unwrap(), 10);
        assert_eq!(comp.read_reg(Regs::DI).unwrap(), 10);
    }
}
