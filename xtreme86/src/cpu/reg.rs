pub struct Reg {
    pub value: u16
}

impl Reg {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn get_low(&self) -> u8 {
        (self.value & 0xFF) as u8
    }

    pub fn get_high(&self) -> u8 {
        ((self.value >> 8) & 0xFF) as u8
    }

    pub fn set_low(&mut self, val: u8) {
        self.value = (self.value & 0xFF00) | ((val as u16) << 8);
    }

    pub fn set_high(&mut self, val: u8) {
        self.value = (self.value & 0x00FF) | (val as u16);
    }
}
