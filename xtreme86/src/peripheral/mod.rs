use crate::cpu::CPU;
use dyn_clone::{DynClone};

pub trait Peripheral : DynClone {
    fn init(&self, comp: &mut CPU, index: usize);
    fn handle_interrupt(&mut self, comp: &mut CPU, int_num: u8) -> usize;
    fn handle_mem_read_byte(&mut self, address: u32) -> u8;
    fn handle_mem_read_word(&mut self, address: u32) -> u16;
    fn handle_mem_write_byte(&mut self, address: u32, val: u8);
    fn handle_mem_write_word(&mut self, address: u32, val: u16);
}

dyn_clone::clone_trait_object!(Peripheral);


