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
        let code = vec![0x00, 0xc6, 0x06, 0x00, 0x00, 0x55];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.read_mem(0), 0x55);
    }

    #[test]
    fn test_add() {
        let code = vec![0x83, 0xc0, 0x5];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        assert_eq!(computer.read_reg(cpu::Regs::AX).unwrap(), 5);
    }
}
