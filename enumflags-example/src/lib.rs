#[cfg(test)]
mod tests {
    use enumflags_derive::FlagBits;

    #[derive(FlagBits)]
    enum Flags {
        Flags1,
        Flags2,
        Flag3
    }

    #[test]
    fn or() {
        let flag = Flags::Flags1;
        assert!()
    }
}
