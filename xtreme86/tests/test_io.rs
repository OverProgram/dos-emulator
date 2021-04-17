use xtreme86::peripheral::Peripheral;
use xtreme86::cpu::{CPU};
use xtreme86::cpu;
use std::fs::File;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(Clone)]
struct TestDevice {
    val: u16,
    part: bool
}

impl Peripheral for TestDevice {
    fn init(&self, comp: &mut CPU, index: usize) {
        comp.hook_io_memory(index, 0xF2);
        comp.hook_io_memory(index, 0xEEDA);

        comp.hook_interrupt(index, 0x12);
    }

    fn handle_interrupt(&mut self,  _: &mut CPU, int_num: u8) -> usize {
        if int_num == 12 {
            let new_part = !self.part;
            self.part = new_part;
        }
        0
    }

    fn handle_mem_read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x00F2 | 0xEEDA => if self.part {
                (self.val & 0x00FF) as u8
            } else {
                (self.val >> 8) as u8
            }
            _ => 0
        }
    }

    fn handle_mem_read_word(&mut self, address: u16) -> u16 {
        match address {
            0x00F2 | 0xEEDA => self.val,
            _ => 0
        }
    }

    fn handle_mem_write_byte(&mut self, address: u16, val: u8) {
        match address {
            0x00F2 | 0xEEDA => if self.part {
                self.val = (self.val & 0xFF00) | (val as u16);
            } else {
                self.val = (self.val & 0x00FF) | ((val as u16) << 8);
            }
            _ => ()
        }
    }

    fn handle_mem_write_word(&mut self, address: u16, val: u16) {
        match address {
            0x00F2 | 0xEEDA => self.val = val,
            _ => ()
        }
    }
}

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

#[test]
fn test_in_out() {

}
