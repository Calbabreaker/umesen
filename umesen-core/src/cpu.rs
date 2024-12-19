use crate::bus::Bus;

bitflags::bitflags! {
    /// Flags for the cpu register
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    struct Flags: u8 {
        const CARRY = 1;
        const ZERO = 1 << 1;
        const INTERRUPT = 1 << 2;
        const DECIMAL = 1 << 3;
        const BREAK = 1 << 4;
        const OVERFLOW = 1 << 5;
        const NEGATIVE = 1 << 6;
    }
}

enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    IndirectIndexed,
}

#[derive(Default)]
pub struct Cpu {
    /// Program counter
    pc: u16,
    // Stack pointer
    sp: u8,
    // Accumulator
    a: u8,
    // Index x register
    x: u8,
    // Index y register
    y: u8,
    flags: Flags,
    bus: Bus,
}

impl Cpu {
    fn read_address(&mut self, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Accumulator => todo!(),
            AddressingMode::Immediate => {
                let value = self.bus.read_byte(self.pc);
                self.pc += 1;
                value
            }
            AddressingMode::ZeroPage => todo!(),
            AddressingMode::ZeroPageX => todo!(),
            AddressingMode::ZeroPageY => todo!(),
            AddressingMode::Relative => todo!(),
            AddressingMode::Absolute => todo!(),
            AddressingMode::AbsoluteX => todo!(),
            AddressingMode::AbsoluteY => todo!(),
            AddressingMode::IndirectX => todo!(),
            AddressingMode::IndirectY => todo!(),
            AddressingMode::IndirectIndexed => todo!(),
        }
    }

    pub fn execute_next(&mut self) {
        let opcode = self.bus.read_byte(self.pc);
        self.pc += 1;
        self.execute(opcode);
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            0xa9 => self.lda(AddressingMode::Immediate),
            0xa5 => self.lda(AddressingMode::ZeroPage),
            0xb5 => self.lda(AddressingMode::ZeroPageX),
            0xad => self.lda(AddressingMode::Absolute),
            0xbd => self.lda(AddressingMode::AbsoluteX),
            0xb9 => self.lda(AddressingMode::AbsoluteY),
            0xa1 => self.lda(AddressingMode::IndirectX),
            0xb1 => self.lda(AddressingMode::IndirectY),
            _ => panic!("Unknown instruction with opcode: {opcode}"),
        }
    }

    fn lda(&mut self, mode: AddressingMode) {
        self.a = self.read_address(mode);
        self.flags.set(Flags::ZERO, self.a == 0);
        self.flags.set(Flags::NEGATIVE, self.a & 0b1000000 != 0);
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::{Cpu, Flags};

    fn execute(rom: &[u8], assert_fn: impl Fn(Cpu)) {
        let mut cpu = Cpu::default();
        for (i, byte) in rom.iter().enumerate() {
            cpu.bus.write_byte(i as u16, *byte);
        }
        cpu.execute_next();
        assert_fn(cpu);
    }

    #[test]
    fn lda() {
        execute(&[0xa9, 123], |cpu| assert_eq!(cpu.a, 123));
        execute(&[0xa9, 0], |cpu| assert_eq!(cpu.flags, Flags::ZERO));
        execute(&[0xa9, 255], |cpu| assert_eq!(cpu.flags, Flags::NEGATIVE));
    }
}
