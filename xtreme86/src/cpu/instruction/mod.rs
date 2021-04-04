use crate::cpu::instruction::opcode::Opcode;

mod opcode;
pub mod actions;
mod data;

pub struct InstructionDecoder {
    opcode_data: [Option<Opcode>; 256],
}

impl InstructionDecoder {
    fn new() -> Self {
        Self {
            opcode_data: data::OPCODE_DATA.clone()
        }
    }
}
