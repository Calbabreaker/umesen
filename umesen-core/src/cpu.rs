use crate::bus::Bus;

bitflags::bitflags! {
    /// Flags for the cpu register
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    struct Flags: u8 {
        const CARRY = 1;
        const ZERO = 1 << 1;
        const INTERRUPT = 1 << 2;
        /// Flag for binary coded decimal where hex 0x00->0x99 is decimal 0->99
        const DECIMAL = 1 << 3;
        const BREAK = 1 << 4;
        const UNUSED = 1 << 5;
        /// Set if arithmetic overflowed 8-bit signed number
        const OVERFLOW = 1 << 6;
        const NEGATIVE = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddressingMode {
    /// Operand contains the value
    Immediate,
    /// Operand contains the address to the value in the first page (256 bytes)
    ZeroPage,
    /// Same as ZeroPage + x register
    ZeroPageX,
    /// Same as ZeroPage + y register
    ZeroPageY,
    /// Same as ZeroPage but now the whole address range (16 bits)
    Absolute,
    /// Same as Absolute + x register
    AbsoluteX,
    /// Same as AbsoluteX but always clock when it would otherwise depend on page cross
    AbsoluteXForceClock,
    /// Same as Absolute + y register
    AbsoluteY,
    AbsoluteYForceClock,
    /// Operand contains the address to the address to the value
    Indirect,
    /// Operand contains the address (with x added) to the address to the value
    IndirectX,
    /// Operand contains the address to the address (with y added) to the value
    IndirectY,
    IndirectYForceClock,
}

#[derive(Default, Clone)]
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
        let opcode = self.read_byte_at_pc();
        self.execute(opcode);
    }

    fn read_byte_at_pc(&mut self) -> u8 {
        self.pc += 1;
        self.bus.read_byte(self.pc - 1)
    }

    fn read_word_at_pc(&mut self) -> u16 {
        self.pc += 2;
        self.bus.read_word(self.pc - 2)
    }

    fn address_add_offset(&mut self, address: u16, offset: u8, mode: AddressingMode) -> u16 {
        let address_added = address.wrapping_add(offset as u16);
        let force_clock = matches!(
            mode,
            AddressingMode::AbsoluteYForceClock
                | AddressingMode::AbsoluteXForceClock
                | AddressingMode::IndirectYForceClock
        );
        if force_clock || address & 0xff00 != address_added & 0xff00 {
            self.bus.clock_cpu();
        }
        address_added
    }

    /// Returns the target address based on the addressing mode and the operand
    fn read_operand_address(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                self.pc += 1;
                self.pc - 1
            }
            AddressingMode::ZeroPage => self.read_byte_at_pc() as u16,
            AddressingMode::ZeroPageX => {
                self.bus.clock_cpu();
                self.read_byte_at_pc().wrapping_add(self.x) as u16
            }
            AddressingMode::ZeroPageY => {
                self.bus.clock_cpu();
                self.read_byte_at_pc().wrapping_add(self.y) as u16
            }
            AddressingMode::Absolute => self.read_word_at_pc(),
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteXForceClock => {
                let address = self.read_word_at_pc();
                self.address_add_offset(address, self.x, mode)
            }
            AddressingMode::AbsoluteY | AddressingMode::AbsoluteYForceClock => {
                let address = self.read_word_at_pc();
                self.address_add_offset(address, self.y, mode)
            }
            AddressingMode::Indirect => {
                let indirect_address = self.read_byte_at_pc();
                self.bus.read_word(indirect_address as u16)
            }
            AddressingMode::IndirectX => {
                let indirect_address = self.read_byte_at_pc().wrapping_add(self.x);
                self.bus.clock_cpu();
                self.bus.read_word(indirect_address as u16)
            }
            AddressingMode::IndirectY | AddressingMode::IndirectYForceClock => {
                let indirect_address = self.read_byte_at_pc();
                let address = self.bus.read_word(indirect_address as u16);
                self.address_add_offset(address, self.y, mode)
            }
        }
    }

    fn read_operand_value(&mut self, mode: AddressingMode) -> u8 {
        let address = self.read_operand_address(mode);
        self.bus.read_byte(address)
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            // Arithmetic
            0x69 => self.adc(AddressingMode::Immediate),
            0x6d => self.adc(AddressingMode::Absolute),
            0x7d => self.adc(AddressingMode::AbsoluteX),
            0x79 => self.adc(AddressingMode::AbsoluteY),
            0x65 => self.adc(AddressingMode::ZeroPage),
            0x75 => self.adc(AddressingMode::ZeroPageX),
            0x61 => self.adc(AddressingMode::IndirectX),
            0x71 => self.adc(AddressingMode::IndirectY),

            0xe9 => self.sbc(AddressingMode::Immediate),
            0xed => self.sbc(AddressingMode::Absolute),
            0xfd => self.sbc(AddressingMode::AbsoluteX),
            0xf9 => self.sbc(AddressingMode::AbsoluteY),
            0xe5 => self.sbc(AddressingMode::ZeroPage),
            0xf5 => self.sbc(AddressingMode::ZeroPageX),
            0xe1 => self.sbc(AddressingMode::IndirectX),
            0xf1 => self.sbc(AddressingMode::IndirectY),

            // Register loads
            0xa9 => self.lda(AddressingMode::Immediate),
            0xa5 => self.lda(AddressingMode::ZeroPage),
            0xb5 => self.lda(AddressingMode::ZeroPageX),
            0xad => self.lda(AddressingMode::Absolute),
            0xbd => self.lda(AddressingMode::AbsoluteX),
            0xb9 => self.lda(AddressingMode::AbsoluteY),
            0xa1 => self.lda(AddressingMode::IndirectX),
            0xb1 => self.lda(AddressingMode::IndirectY),

            0xa2 => self.ldx(AddressingMode::Immediate),
            0xae => self.ldx(AddressingMode::Absolute),
            0xbe => self.ldx(AddressingMode::AbsoluteY),
            0xa6 => self.ldx(AddressingMode::ZeroPage),
            0xb6 => self.ldx(AddressingMode::ZeroPageY),

            0xa0 => self.ldy(AddressingMode::Immediate),
            0xac => self.ldy(AddressingMode::Absolute),
            0xbc => self.ldy(AddressingMode::AbsoluteX),
            0xa4 => self.ldy(AddressingMode::ZeroPage),
            0xb4 => self.ldy(AddressingMode::ZeroPageX),

            // Register stores
            0x8d => self.sta(AddressingMode::Absolute),
            0x9d => self.sta(AddressingMode::AbsoluteXForceClock),
            0x99 => self.sta(AddressingMode::AbsoluteYForceClock),
            0x85 => self.sta(AddressingMode::ZeroPage),
            0x95 => self.sta(AddressingMode::ZeroPageX),
            0x81 => self.sta(AddressingMode::IndirectX),
            0x91 => self.sta(AddressingMode::IndirectYForceClock),

            0x8e => self.stx(AddressingMode::Absolute),
            0x86 => self.stx(AddressingMode::ZeroPage),
            0x96 => self.stx(AddressingMode::ZeroPageY),

            0x8c => self.sty(AddressingMode::Absolute),
            0x84 => self.sty(AddressingMode::ZeroPage),
            0x94 => self.sty(AddressingMode::ZeroPageX),

            // Register transfers
            0xaa => self.tax(),
            0xa8 => self.tay(),
            0xba => self.tsx(),
            0x8a => self.txa(),
            0x9a => self.txs(),
            0x98 => self.tya(),

            _ => panic!("Unknown instruction with opcode: 0x{opcode:02x}"),
        }
    }

    fn set_zero_neg_flags(&mut self, a: u8) {
        self.flags.set(Flags::ZERO, a == 0);
        self.flags.set(Flags::NEGATIVE, a & 0b1000_0000 != 0);
    }

    fn set_overflow_flag(&mut self, a: u8, adder: u8, result: u8) {
        let adder_same_sign = (a ^ adder) & 0b1000_0000 == 0;
        let result_changed_sign = (a ^ result) & 0b1000_0000 != 0;
        self.flags
            .set(Flags::OVERFLOW, adder_same_sign && result_changed_sign);
    }

    fn do_adc(&mut self, adder: u8, carry: bool) {
        let mut result_word = adder as u16 + self.a as u16 + carry as u16;

        if self.flags.contains(Flags::DECIMAL) {
            // Account for adding going into between 0xa and 0xf
            if (self.a & 0xf) + (adder & 0xf) + carry as u8 > 9 {
                result_word += 0x06;
            }
            // Account for adding going into between 0xa0 and 0xff
            if result_word > 0x99 {
                result_word += 0x60;
            }
        }

        let result = result_word as u8;
        self.flags.set(Flags::CARRY, result_word > 0xff);
        self.set_overflow_flag(self.a, adder, result);
        self.set_zero_neg_flags(result);
        self.a = result;
    }

    fn adc(&mut self, mode: AddressingMode) {
        let adder = self.read_operand_value(mode);
        self.do_adc(adder, self.flags.contains(Flags::CARRY));
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let adder = self.read_operand_value(mode);
        // Inverting results in inverting the sign so the adc can be resued
        self.do_adc(!adder, !self.flags.contains(Flags::CARRY));
    }

    fn lda(&mut self, mode: AddressingMode) {
        self.a = self.read_operand_value(mode);
        self.set_zero_neg_flags(self.a);
    }

    fn ldx(&mut self, mode: AddressingMode) {
        self.x = self.read_operand_value(mode);
        self.set_zero_neg_flags(self.x);
    }

    fn ldy(&mut self, mode: AddressingMode) {
        self.y = self.read_operand_value(mode);
        self.set_zero_neg_flags(self.y);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, self.a);
    }

    fn stx(&mut self, mode: AddressingMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, self.a);
    }

    fn sty(&mut self, mode: AddressingMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, self.a);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.set_zero_neg_flags(self.x);
    }

    fn tay(&self) {
        todo!()
    }

    fn tsx(&self) {
        todo!()
    }

    fn txa(&self) {
        todo!()
    }

    fn txs(&self) {
        todo!()
    }

    fn tya(&self) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::{Cpu, Flags};

    fn execute(rom: &[u8], assert_fn: impl Fn(Cpu)) {
        let mut cpu = Cpu {
            a: 0xff,
            x: 0xff,
            y: 0xff,
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
    fn addressing_modes() {
        // Immediate mode lda
        execute(&[0xa9, 123], |cpu| {
            assert_eq!(cpu.a, 123);
            assert_eq!(cpu.bus.cpu_cycles, 2);
            assert_eq!(cpu.pc, 2);
        });

        // Zero page mode lda
        execute(&[0xa5, 0x02, 69], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 3);
            assert_eq!(cpu.pc, 2);
        });

        // Zero page x mode lda
        execute(&[0xb5, 0x03, 69], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 4);
            assert_eq!(cpu.pc, 2);
        });

        // Zero page y mode ldx
        execute(&[0xb6, 0x03, 69], |cpu| {
            assert_eq!(cpu.x, 69);
            assert_eq!(cpu.bus.cpu_cycles, 4);
            assert_eq!(cpu.pc, 2);
        });

        // Absolute mode lda
        execute(&[0xad, 0x32, 0x01], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 4);
            assert_eq!(cpu.pc, 3);
        });

        // Absolute mode x lda
        execute(&[0xbd, 0x33, 0x00], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 5);
            assert_eq!(cpu.pc, 3);
        });

        // Absolute mode x lda (no page cross)
        execute(&[0xbd], |cpu| assert_eq!(cpu.bus.cpu_cycles, 4));

        // Absolute mode x sta (always extra clock)
        execute(&[0x9d, 0x12, 0x00], |mut cpu| {
            assert_eq!(cpu.bus.cpu_cycles, 5);
            assert_eq!(cpu.bus.read_byte(0x111), 0xff);
        });

        // Absolute mode y lda
        execute(&[0xb9, 0x33, 0x00], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 5);
            assert_eq!(cpu.pc, 3);
        });

        // Indirect x mode lda
        execute(&[0xa1, 0x04, 0, 0x32, 0x01], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 6);
            assert_eq!(cpu.pc, 2);
        });

        // Indirect y mode lda
        execute(&[0xb1, 0x03, 0, 0x33], |cpu| {
            assert_eq!(cpu.a, 69);
            assert_eq!(cpu.bus.cpu_cycles, 6);
            assert_eq!(cpu.pc, 2);
        });
    }

    #[test]
    fn zero_neg_flags() {
        // lda immediate
        execute(&[0xa9, 22], |cpu| assert_eq!(cpu.flags, Flags::empty()));
        execute(&[0xa9, 0], |cpu| assert_eq!(cpu.flags, Flags::ZERO));
        execute(&[0xa9, 128], |cpu| assert_eq!(cpu.flags, Flags::NEGATIVE));
    }

    #[test]
    fn adc() {
        execute(&[0x69, 69, 0x69, 69, 0x69, 0x69], |mut cpu| {
            assert_eq!(cpu.a, 68);
            assert_eq!(cpu.flags, Flags::CARRY);
            cpu.execute_next();
            assert_eq!(cpu.a, 138);
            assert_eq!(cpu.flags, Flags::OVERFLOW | Flags::NEGATIVE);
            cpu.flags.set(Flags::DECIMAL, true);
            cpu.execute_next();
            assert_eq!(cpu.a, 0x59);
            assert_eq!(cpu.flags, Flags::CARRY | Flags::DECIMAL);
        });
    }

    #[test]
    fn sbc() {
        execute(&[0xe9, 69, 0xe9, 0x69], |cpu| {
            assert_eq!(cpu.a, 186);
            assert_eq!(cpu.flags, Flags::CARRY | Flags::NEGATIVE);
        });
    }

    #[test]
    fn lda() {
        execute(&[0xa9, 123], |cpu| assert_eq!(cpu.a, 123));
    }

    #[test]
    fn ldx() {
        execute(&[0xa2, 69], |cpu| assert_eq!(cpu.x, 69));
    }

    #[test]
    fn ldy() {
        execute(&[0xa0, 69], |cpu| assert_eq!(cpu.y, 69));
    }

    #[test]
    fn sta() {
        execute(&[0x85, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), 0xff));
    }

    #[test]
    fn stx() {
        execute(&[0x86, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), 0xff));
    }

    #[test]
    fn sty() {
        execute(&[0x84, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), 0xff));
    }
}
