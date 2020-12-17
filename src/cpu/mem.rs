use super::{CPU};

impl CPU {
    pub fn mov(&mut self) -> usize {
        self.write_to_arg(self.dst.clone().unwrap(), self.src.clone().unwrap()).unwrap();
        0
    }

    pub fn nop(&mut self) -> usize {
        0
    }
}
