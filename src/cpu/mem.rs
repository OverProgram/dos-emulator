use super::{CPU};

impl CPU {
    pub fn mov(&mut self) -> usize {
        self.write_to_arg(self.dst.clone().unwrap(), self.src.clone().unwrap());
        0
    }
}
