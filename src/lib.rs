mod cpu;

#[cfg(test)]
mod tests {
    use super::cpu;
    use crate::cpu::Regs;

    fn new_cpu_vec(code: Vec<u8>) -> cpu::CPU {
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer
    }

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
        let code = vec![0x66, 0xc6, 0x6, 0x0, 0x0, 0x0];
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

    #[test]
    fn test_sub() {
        let code = vec![0x30, 0x80, 0x2e, 0x0, 0x0, 0x20];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.read_mem(0), 0x10);
    }

    #[test]
    fn test_and() {
        let code = vec![0xb8, 0xFF, 0x0, 0xbb, 0xaa, 0x0, 0x21, 0xc3];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        computer.execute_next();
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::BX).unwrap(), 0xAA);
    }

    #[test]
    fn test_or() {
        let code = vec![0xaa, 0x00, 0xb9, 0x55, 0x0, 0xb, 0xe, 0x0, 0x0];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(2);
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::CX).unwrap(), 0xFF);
    }

    #[test]
    fn test_inc_reg() {
        let code = vec![0x40];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next();
        assert_eq!(computer.read_reg(Regs::AX).unwrap(), 0x01);
    }

    #[test]
    fn test_dec_mem() {
        let code = vec![0xff, 0xfe, 0xe, 0x0, 0x0];
        let mut computer = cpu::CPU::new(code.len());
        computer.load(code, 0);
        computer.execute_next_from(1);
        assert_eq!(computer.read_mem(0), 0xFE)
    }

    #[test]
    fn test_carry() {
        let mut computer = new_cpu_vec(vec![0xff, 0xfe, 0x6, 0x0, 0x0]);
        computer.execute_next_from(1);
        assert_eq!(computer.read_reg(Regs::FLAGS).unwrap() & (cpu::CPUFlags::CARRY | cpu::CPUFlags::ZERO), (cpu::CPUFlags::CARRY | cpu::CPUFlags::ZERO));
    }
}
