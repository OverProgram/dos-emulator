use super::{CPU};

impl CPU {
    pub fn add(&mut self) -> usize {
        let sum = self.operation_2_args(|src, dst| src + dst, |src, dst| src + dst);
        self.write_to_arg(self.dst.clone().unwrap(), sum);
        0
    }

    pub fn sub(&mut self) -> usize {
        let dif = self.operation_2_args(|src, dst| dst - src, |src, dst| dst - src);
        self.write_to_arg(self.dst.clone().unwrap(), dif);
        0
    }

    pub fn inc(&mut self) {
        let sum = self.operation_1_arg(|dst| dst + 1, |dst| dst + 1);
        self.write_to_arg(self.dst.clone().unwrap(), sum);
    }

    pub fn dec(&mut self) {
        let sum = self.operation_1_arg(|dst| dst - 1, |dst| dst - 1);
        self.write_to_arg(self.dst.clone().unwrap(), sum);
    }
}
