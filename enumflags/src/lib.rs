pub mod container;

use std::ops;
use crate::container::FlagContainer;

pub trait FlagBits {
    type Container: container::FlagContainer + std::ops::BitOr<Output = Self::Container> +
    std::ops::BitAnd<Output = Self::Container> + std::ops::Not<Output = Self::Container>;

    fn to_loc(&self) -> <<Self as FlagBits>::Container as container::FlagContainer>::Index;
}

#[derive(Clone)]
pub enum FlagBit<T: FlagBits> {
    Is(T),
    Not(T)
}

impl<T: FlagBits> ops::Not for FlagBit<T> {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            FlagBit::Is(val) => Self::Not(val),
            FlagBit::Not(val) => Self::Is(val)
        }
    }
}

pub struct Flags<Flag: FlagBits> {
    flags: Flag::Container
}

impl<Flag: FlagBits> Flags<Flag> {
    pub fn new() -> Self {
        Self {
            flags: Flag::Container::none()
        }
    }

    pub fn has(&self, flag: Flag) -> bool {
        self.flags.has(flag.to_loc())
    }

    pub fn add(&mut self, flag: FlagBit<Flag>) {
        match flag {
            FlagBit::Is(val) => self.flags.add(val.to_loc()),
            FlagBit::Not(val) => self.flags.remove(val.to_loc())
        }
    }
}

impl<Flag: FlagBits> ops::BitOr for Flags<Flag> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            flags: self.flags | rhs.flags
        }
    }
}

impl<Flag: FlagBits> ops::BitOr<FlagBit<Flag>> for Flags<Flag> {
    type Output = Self;

    fn bitor(self, rhs: FlagBit<Flag>) -> Self::Output {
        Self {
            flags: {
                let mut tmp = Self { flags: self.flags };
                tmp.add(rhs);
                tmp.flags
            }
        }
    }
}

impl<Flag: FlagBits> ops::BitAnd for Flags<Flag> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            flags: self.flags & rhs.flags
        }
    }
}

impl<Flag: FlagBits> ops::BitAnd<FlagBit<Flag>> for Flags<Flag> {
    type Output = Self;

    fn bitand(self, rhs: FlagBit<Flag>) -> Self::Output {
        Self {
            flags: {
                let mut tmp = Self { flags: self.flags };
                tmp.add(!rhs);
                tmp.flags
            }
        }
    }
}

impl<Flag: FlagBits> ops::Not for Flags<Flag> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            flags: !self.flags
        }
    }
}
