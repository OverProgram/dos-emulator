use crate::cpu::instruction::opcode::Opcode;

mod opcode;
mod actions;

pub struct InstructionDecoder {
    opcode_data: [Opcode; u8::MAX as usize],
}

impl InstructionDecoder {
    fn new() -> Self {
        Self {
            opcode_data: Opcode::get_opcode_data()
        }
    }
}
