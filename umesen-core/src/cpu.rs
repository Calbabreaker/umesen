use crate::{bus::Bus, CpuError};

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

/// Addressing modes (most of them) for instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddrMode {
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
    pub bus: Bus,
}

macro_rules! load_reg {
    ($this: ident, $reg: ident, $operator: tt, $mode: expr) => {{
        $this.$reg $operator $this.read_operand_value($mode);
        $this.set_zero_neg_flags($this.$reg);
    }};
}

macro_rules! set_reg {
    ($this: ident, $reg: ident, $operator: tt, $value: expr) => {{
        $this.$reg $operator $value;
        $this.bus.clock_cpu();
        $this.set_zero_neg_flags($this.$reg);
    }};
}

impl Cpu {
    pub fn execute_next(&mut self) -> Result<(), CpuError> {
        let opcode = self.read_byte_at_pc();
        self.execute(opcode)
    }

    fn read_byte_at_pc(&mut self) -> u8 {
        self.pc += 1;
        self.bus.read_byte(self.pc - 1)
    }

    fn read_word_at_pc(&mut self) -> u16 {
        self.pc += 2;
        self.bus.read_word(self.pc - 2)
    }

    fn address_add_offset(&mut self, address: u16, offset: u8, mode: AddrMode) -> u16 {
        let address_added = address.wrapping_add(offset as u16);
        let force_clock = matches!(
            mode,
            AddrMode::AbsoluteYForceClock
                | AddrMode::AbsoluteXForceClock
                | AddrMode::IndirectYForceClock
        );
        if force_clock || address & 0xff00 != address_added & 0xff00 {
            self.bus.clock_cpu();
        }
        address_added
    }

    /// Returns the target address based on the addressing mode and the operand
    fn read_operand_address(&mut self, mode: AddrMode) -> u16 {
        match mode {
            AddrMode::Immediate => {
                self.pc += 1;
                self.pc - 1
            }
            AddrMode::ZeroPage => self.read_byte_at_pc() as u16,
            AddrMode::ZeroPageX => {
                self.bus.clock_cpu();
                self.read_byte_at_pc().wrapping_add(self.x) as u16
            }
            AddrMode::ZeroPageY => {
                self.bus.clock_cpu();
                self.read_byte_at_pc().wrapping_add(self.y) as u16
            }
            AddrMode::Absolute => self.read_word_at_pc(),
            AddrMode::AbsoluteX | AddrMode::AbsoluteXForceClock => {
                let address = self.read_word_at_pc();
                self.address_add_offset(address, self.x, mode)
            }
            AddrMode::AbsoluteY | AddrMode::AbsoluteYForceClock => {
                let address = self.read_word_at_pc();
                self.address_add_offset(address, self.y, mode)
            }
            AddrMode::Indirect => {
                let indirect_address = self.read_byte_at_pc();
                self.bus.read_word(indirect_address as u16)
            }
            AddrMode::IndirectX => {
                let indirect_address = self.read_byte_at_pc().wrapping_add(self.x);
                self.bus.clock_cpu();
                self.bus.read_word(indirect_address as u16)
            }
            AddrMode::IndirectY | AddrMode::IndirectYForceClock => {
                let indirect_address = self.read_byte_at_pc();
                let address = self.bus.read_word(indirect_address as u16);
                self.address_add_offset(address, self.y, mode)
            }
        }
    }

    fn read_operand_value(&mut self, mode: AddrMode) -> u8 {
        let address = self.read_operand_address(mode);
        self.bus.read_byte(address)
    }

    fn execute(&mut self, opcode: u8) -> Result<(), CpuError> {
        match opcode {
            // -- Arithmetic --
            0x69 => self.adc(AddrMode::Immediate),
            0x65 => self.adc(AddrMode::ZeroPage),
            0x75 => self.adc(AddrMode::ZeroPageX),
            0x6d => self.adc(AddrMode::Absolute),
            0x7d => self.adc(AddrMode::AbsoluteX),
            0x79 => self.adc(AddrMode::AbsoluteY),
            0x61 => self.adc(AddrMode::IndirectX),
            0x71 => self.adc(AddrMode::IndirectY),

            0xe9 => self.sbc(AddrMode::Immediate),
            0xe5 => self.sbc(AddrMode::ZeroPage),
            0xf5 => self.sbc(AddrMode::ZeroPageX),
            0xed => self.sbc(AddrMode::Absolute),
            0xfd => self.sbc(AddrMode::AbsoluteX),
            0xf9 => self.sbc(AddrMode::AbsoluteY),
            0xe1 => self.sbc(AddrMode::IndirectX),
            0xf1 => self.sbc(AddrMode::IndirectY),

            // -- Register loads --
            // lda
            0xa9 => load_reg!(self, a, =, AddrMode::Immediate),
            0xa5 => load_reg!(self, a, =, AddrMode::ZeroPage),
            0xb5 => load_reg!(self, a, =, AddrMode::ZeroPageX),
            0xad => load_reg!(self, a, =, AddrMode::Absolute),
            0xbd => load_reg!(self, a, =, AddrMode::AbsoluteX),
            0xb9 => load_reg!(self, a, =, AddrMode::AbsoluteY),
            0xa1 => load_reg!(self, a, =, AddrMode::IndirectX),
            0xb1 => load_reg!(self, a, =, AddrMode::IndirectY),

            // ldx
            0xa2 => load_reg!(self, x, =, AddrMode::Immediate),
            0xa6 => load_reg!(self, x, =, AddrMode::ZeroPage),
            0xb6 => load_reg!(self, x, =, AddrMode::ZeroPageY),
            0xae => load_reg!(self, x, =, AddrMode::Absolute),
            0xbe => load_reg!(self, x, =, AddrMode::AbsoluteY),

            // ldy
            0xa0 => load_reg!(self, y, =, AddrMode::Immediate),
            0xa4 => load_reg!(self, y, =, AddrMode::ZeroPage),
            0xb4 => load_reg!(self, y, =, AddrMode::ZeroPageX),
            0xac => load_reg!(self, y, =, AddrMode::Absolute),
            0xbc => load_reg!(self, y, =, AddrMode::AbsoluteX),

            // -- Register stores --
            // sta
            0x85 => self.store(self.a, AddrMode::ZeroPage),
            0x95 => self.store(self.a, AddrMode::ZeroPageX),
            0x8d => self.store(self.a, AddrMode::Absolute),
            0x9d => self.store(self.a, AddrMode::AbsoluteXForceClock),
            0x99 => self.store(self.a, AddrMode::AbsoluteYForceClock),
            0x81 => self.store(self.a, AddrMode::IndirectX),
            0x91 => self.store(self.a, AddrMode::IndirectYForceClock),

            // stx
            0x8e => self.store(self.x, AddrMode::Absolute),
            0x86 => self.store(self.x, AddrMode::ZeroPage),
            0x96 => self.store(self.x, AddrMode::ZeroPageY),

            // st
            0x8c => self.store(self.y, AddrMode::Absolute),
            0x84 => self.store(self.y, AddrMode::ZeroPage),
            0x94 => self.store(self.y, AddrMode::ZeroPageX),

            // -- Register transfers --
            0xaa => set_reg!(self, x,  =, self.a),  // tax
            0xa8 => set_reg!(self, y,  =, self.a),  // tay
            0xba => set_reg!(self, x,  =, self.sp), // tsx
            0x8a => set_reg!(self, a,  =, self.x),  // txa
            0x9a => set_reg!(self, sp, =, self.x),  // txs
            0x98 => set_reg!(self, a,  =, self.y),  // tya

            // -- Flag clear and set --
            0x18 => self.set_flag(Flags::CARRY, false), // clc
            0xd8 => self.set_flag(Flags::DECIMAL, false), // cld
            0x58 => self.set_flag(Flags::INTERRUPT, false), // cli
            0xb8 => self.set_flag(Flags::OVERFLOW, false), // clv
            0x38 => self.set_flag(Flags::CARRY, true),  // sec
            0xf8 => self.set_flag(Flags::DECIMAL, true), // sed
            0x78 => self.set_flag(Flags::INTERRUPT, true), // sei

            // -- Logic --
            // and
            0x29 => load_reg!(self, a, &=, AddrMode::Immediate),
            0x25 => load_reg!(self, a, &=, AddrMode::ZeroPage),
            0x35 => load_reg!(self, a, &=, AddrMode::ZeroPageX),
            0x2d => load_reg!(self, a, &=, AddrMode::Absolute),
            0x3d => load_reg!(self, a, &=, AddrMode::AbsoluteX),
            0x39 => load_reg!(self, a, &=, AddrMode::AbsoluteY),
            0x21 => load_reg!(self, a, &=, AddrMode::IndirectX),
            0x31 => load_reg!(self, a, &=, AddrMode::IndirectY),

            0x2c => self.bit(AddrMode::Absolute),
            0x24 => self.bit(AddrMode::ZeroPage),

            // eor
            0x49 => load_reg!(self, a, ^=, AddrMode::Immediate),
            0x45 => load_reg!(self, a, ^=, AddrMode::ZeroPage),
            0x55 => load_reg!(self, a, ^=, AddrMode::ZeroPageX),
            0x4d => load_reg!(self, a, ^=, AddrMode::Absolute),
            0x5d => load_reg!(self, a, ^=, AddrMode::AbsoluteX),
            0x59 => load_reg!(self, a, ^=, AddrMode::AbsoluteY),
            0x41 => load_reg!(self, a, ^=, AddrMode::IndirectX),
            0x51 => load_reg!(self, a, ^=, AddrMode::IndirectY),

            // ora
            0x09 => load_reg!(self, a, |=, AddrMode::Immediate),
            0x05 => load_reg!(self, a, |=, AddrMode::ZeroPage),
            0x15 => load_reg!(self, a, |=, AddrMode::ZeroPageX),
            0x0d => load_reg!(self, a, |=, AddrMode::Absolute),
            0x1d => load_reg!(self, a, |=, AddrMode::AbsoluteX),
            0x19 => load_reg!(self, a, |=, AddrMode::AbsoluteY),
            0x01 => load_reg!(self, a, |=, AddrMode::IndirectX),
            0x11 => load_reg!(self, a, |=, AddrMode::IndirectY),

            // cmp
            0xc9 => self.compare(self.a, AddrMode::Immediate),
            0xc5 => self.compare(self.a, AddrMode::ZeroPage),
            0xd5 => self.compare(self.a, AddrMode::ZeroPageX),
            0xcd => self.compare(self.a, AddrMode::Absolute),
            0xdd => self.compare(self.a, AddrMode::AbsoluteX),
            0xd9 => self.compare(self.a, AddrMode::AbsoluteY),
            0xc1 => self.compare(self.a, AddrMode::IndirectX),
            0xd1 => self.compare(self.a, AddrMode::IndirectY),

            // cpx
            0xe0 => self.compare(self.x, AddrMode::Immediate),
            0xe4 => self.compare(self.x, AddrMode::ZeroPage),
            0xec => self.compare(self.x, AddrMode::Absolute),

            // cpy
            0xc0 => self.compare(self.y, AddrMode::Immediate),
            0xc4 => self.compare(self.y, AddrMode::ZeroPage),
            0xcc => self.compare(self.y, AddrMode::Absolute),

            // Does nothing nop
            0xea => self.bus.clock_cpu(),

            _ => return Err(CpuError::UnknownOpcode(opcode)),
        };
        Ok(())
    }

    fn set_zero_neg_flags(&mut self, a: u8) {
        self.flags.set(Flags::ZERO, a == 0);
        self.flags.set(Flags::NEGATIVE, a & 0b1000_0000 != 0);
    }

    // Set overflow if the resulting addition overflowed a 8-bit number with 2's compliment
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

    fn adc(&mut self, mode: AddrMode) {
        let adder = self.read_operand_value(mode);
        self.do_adc(adder, self.flags.contains(Flags::CARRY));
    }

    fn sbc(&mut self, mode: AddrMode) {
        let adder = self.read_operand_value(mode);
        // Inverting results in inverting the sign so the adc can be resued
        self.do_adc(!adder, !self.flags.contains(Flags::CARRY));
    }

    fn store(&mut self, register: u8, mode: AddrMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, register);
    }

    fn set_flag(&mut self, flag: Flags, value: bool) {
        self.flags.set(flag, value);
        self.bus.clock_cpu();
    }

    fn bit(&mut self, mode: AddrMode) {
        let value = self.read_operand_value(mode);
        let result = self.a & value;
        self.flags.set(Flags::ZERO, result == 0);
        self.flags.set(Flags::OVERFLOW, value & (0b0100_0000) != 0);
        self.flags.set(Flags::NEGATIVE, value & (0b1000_0000) != 0);
    }

    fn compare(&mut self, register: u8, mode: AddrMode) {
        let value = self.read_operand_value(mode);
        let result = register - value;
        self.flags.set(Flags::CARRY, register >= value);
        self.set_zero_neg_flags(result);
    }
}

#[cfg(test)]
#[path = "cpu.test.rs"]
mod test;
