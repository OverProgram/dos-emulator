mod cpu;

#[cfg(test)]
mod tests {
    use super::cpu;

    #[test]
    fn test_mov_reg_shorthand() {
        let code = vec![0xB8, 0x06, 0x00];    // mov ax, 0x6
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        assert_eq!(computer.read_reg(cpu::Regs::AX).unwrap(), 6);
    }

    #[test]
    fn test_mov_reg() {
        let code = vec![0x66, 0x8B, 0x1D, 0x00, 0x00, 0x00, 0x00];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.read_mem(0), 0x0);
    }

    #[test]
    fn test_mov_ptr() {
        let code = vec![0x00, 0xC6, 0x05, 0x00, 0x00, 0x55];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.read_mem(0), 0x55);
    }

    #[test]
    fn test_mov_sib() {
        let code = vec![0xAA, 0xAA, 0xB8, 0x01, 0x00, 0xBB, 0x01, 0x00, 0x89, 0x4C, 0x58, 0xFD];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(2);
        computer.execute_next();
        computer.execute_next();
        assert_eq!(computer.read_reg(cpu::Regs::CX).unwrap(), 0xAAAA);
    }

    fn test_add() {

    }
}
