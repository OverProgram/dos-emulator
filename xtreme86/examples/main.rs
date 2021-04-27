use xtreme86::peripheral::Peripheral;
use xtreme86::cpu::{CPU, Regs};
use std::path::{Path};
use std::fs::File;
use std::fs;
use std::io::Read;

#[derive(Clone)]
struct Printer;

impl Peripheral for Printer {
    fn init(&self, comp: &mut CPU, index: usize) {
        comp.hook_interrupt(index, 0x21);
    }

    fn handle_interrupt(&mut self, comp: &mut CPU, _: u8) -> usize {
        let mut ptr = comp.read_reg(Regs::DX).unwrap();
        let mut string = Vec::<u8>::new();
        let mut byte = comp.probe_mem_ds(ptr);
        while byte != 0 {
            string.push(byte);
            ptr += 1;
            byte = comp.probe_mem_ds(ptr);
        }

        print!("{}", std::str::from_utf8(string.as_slice()).unwrap());

        0
    }

    fn handle_mem_read_byte(&mut self, _: u16) -> u8 { 0 }
    fn handle_mem_read_word(&mut self, _: u16) -> u16 { 0 }
    fn handle_mem_write_byte(&mut self, _: u16, _: u8) {}
    fn handle_mem_write_word(&mut self, _: u16, _: u16) {}
}

fn load_file(path: &Path, comp: &mut CPU) {
    let mut f = File::open(path).unwrap();
    let metadata = fs::metadata(&path).unwrap();
    let mut buf = vec![0u8; metadata.len() as usize];

    f.read(&mut buf).unwrap();

    comp.load(buf, 0x103F0);
}

fn main() {
    let mut comp = CPU::new(0x7FFFFF);
    comp.set_reg(Regs::SS, 0x003F);
    comp.set_reg(Regs::CS, 0x103F);
    comp.set_reg(Regs::DS, 0x203F);
    comp.set_reg(Regs::SP, 0xFFFF);
    comp.set_reg(Regs::IP, 0x0000);

    load_file(Path::new("examples/print.out"), &mut comp);
    comp.hook_peripheral(Box::new(Printer));

    let string = "Hello, world\0";
    comp.write_bytes(CPU::physical_address(0x4000, 0) as usize, Vec::from(string)).unwrap();

    comp.run_to_nop_from_ip();
}
