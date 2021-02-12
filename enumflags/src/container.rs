use std::ops;

pub trait FlagContainer {
    type Index: ops::Shl<Output = Self::Index> + ops::Not<Output = Self::Index>;

    fn none() -> Self;
    fn identity() -> Self::Index;
    fn add(&mut self, bit: Self::Index);
    fn remove(&mut self, bit: Self::Index);
    fn has(&self, bit: Self::Index) -> bool;
}

impl FlagContainer for usize {
    type Index = Self;

    fn none() -> Self {
        0
    }

    fn identity() -> Self::Index {
        1
    }

    fn add(&mut self, bit: Self::Index) {
        *self |= bit;
    }

    fn remove(&mut self, bit: Self::Index) {
        *self &= !bit;
    }

    fn has(&self, bit: Self::Index) -> bool {
        self & (bit) > 0
    }
}

impl FlagContainer for u8 {
    type Index = Self;

    fn none() -> Self::Index {
        0
    }

    fn identity() -> Self::Index {
        1
    }

    fn add(&mut self, bit: Self::Index) {
        *self |= bit;
    }

    fn remove(&mut self, bit: Self::Index) {
        *self &= !bit;
    }

    fn has(&self, bit: Self::Index) -> bool {
        self & (bit) > 0
    }
}

impl FlagContainer for u16 {
    type Index = Self;

    fn none() -> Self::Index {
        0
    }

    fn identity() -> Self::Index {
        1
    }

    fn add(&mut self, bit: Self::Index) {
        *self |= bit;
    }

    fn remove(&mut self, bit: Self::Index) {
        *self &= !bit;
    }

    fn has(&self, bit: Self::Index) -> bool {
        self & (bit) > 0
    }
}

impl FlagContainer for u32 {
    type Index = Self;

    fn none() -> Self::Index {
        0
    }

    fn identity() -> Self::Index {
        1
    }

    fn add(&mut self, bit: Self::Index) {
        *self |= bit;
    }

    fn remove(&mut self, bit: Self::Index) {
        *self &= !bit;
    }

    fn has(&self, bit: Self::Index) -> bool {
        self & (bit) > 0
    }
}

impl FlagContainer for u64 {
    type Index = Self;

    fn none() -> Self::Index {
        0
    }

    fn identity() -> Self::Index {
        1
    }

    fn add(&mut self, bit: Self::Index) {
        *self |= bit;
    }

    fn remove(&mut self, bit: Self::Index) {
        *self &= !bit;
    }

    fn has(&self, bit: Self::Index) -> bool {
        self & (bit) > 0
    }
}
