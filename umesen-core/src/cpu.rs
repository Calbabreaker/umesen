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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddressingMode {
    Accumulator,
    /// The next byte contains the value
    Immediate,
    /// The next byte contains the address to the value in the first page (256 bytes)
    ZeroPage,
    /// Same as ZeroPage + x register
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
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
    pub fn execute_next(&mut self) {
        let opcode = self.read_at_pc();
        self.execute(opcode);
    }

    fn read_at_pc(&mut self) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.bus.read_byte(pc)
    }

    fn read_word_at_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 2;
        self.bus.read_word(pc)
    }

    fn read_address(&mut self, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Accumulator => todo!(),
            AddressingMode::Immediate => self.read_at_pc(),
            AddressingMode::ZeroPage => {
                let address = self.read_at_pc();
                self.bus.read_byte(address as u16)
            }
            AddressingMode::ZeroPageX => {
                let address = self.read_at_pc().wrapping_add(self.x);
                self.bus.clock_cpu();
                self.bus.read_byte(address as u16)
            }
            AddressingMode::ZeroPageY => todo!(),
            AddressingMode::Relative => todo!(),
            AddressingMode::Absolute => {
                let address = self.read_word_at_pc();
                self.bus.read_byte(address)
            }
            AddressingMode::AbsoluteX => {
                let base_address = self.read_word_at_pc();
                let address_x = base_address.wrapping_add(self.x as u16);
                // Check if page is crossed
                if base_address >> 8 != address_x >> 8 {
                    self.bus.clock_cpu();
                }
                self.bus.read_byte(address_x)
            }
            AddressingMode::AbsoluteY => todo!(),
            AddressingMode::IndirectX => todo!(),
            AddressingMode::IndirectY => todo!(),
        }
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

            0xa2 => self.ldx(AddressingMode::Immediate),
            _ => panic!("Unknown instruction with opcode: {opcode}"),
        }
    }

    fn set_zero_neg_flags(&mut self, x: u8) {
        self.flags.set(Flags::ZERO, x == 0);
        self.flags.set(Flags::NEGATIVE, x & 0b1000000 != 0);
    }

    fn lda(&mut self, mode: AddressingMode) {
        self.a = self.read_address(mode);
        self.set_zero_neg_flags(self.a);
    }

    fn ldx(&mut self, mode: AddressingMode) {
        self.x = self.read_address(mode);
        self.set_zero_neg_flags(self.x);
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::{Cpu, Flags};

    fn execute(rom: &[u8], assert_fn: impl Fn(Cpu)) {
        let mut cpu = Cpu {
            x: 0xff,
            ..Default::default()
        };
        cpu.bus.ram[0x132] = 69;
        for (i, x) in rom.iter().enumerate() {
            cpu.bus.ram[i] = *x;
        }
        cpu.execute_next();
        assert_fn(cpu);
    }

    #[test]
    fn lda() {
        // Immediate mode
        execute(&[0xa9, 123], |cpu| {
            assert_eq!(cpu.a, 123);
            assert_eq!(cpu.bus.cpu_cycles, 2);
            assert_eq!(cpu.pc, 2);
        });
        execute(&[0xa9, 0], |cpu| assert_eq!(cpu.flags, Flags::ZERO));
        execute(&[0xa9, 255], |cpu| assert_eq!(cpu.flags, Flags::NEGATIVE));

        // Zero page mode
        execute(&[0xa5, 0x02, 69], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 3);
            assert_eq!(cpu.pc, 2);
        });

        // Zero page + x mode
        execute(&[0xb5, 0x03, 69], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 4);
            assert_eq!(cpu.pc, 2);
        });

        // Absolute mode
        execute(&[0xad, 0x32, 0x01], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 4);
            assert_eq!(cpu.pc, 3);
        });

        // Absolute mode + x
        execute(&[0xbd, 0x33, 0x00], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 5);
            assert_eq!(cpu.pc, 3);
        });
    }

    #[test]
    fn ldx() {
        execute(&[0xa2, 69], |cpu| assert_eq!(cpu.x, 69));
    }
}
