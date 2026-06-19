mod bus;
mod disassembler;
mod opcode;

use bus::CpuBus;
pub use disassembler::Disassembler;
pub use opcode::{AddrMode, Opcode};

/// Number of clock cycles per second
pub const CLOCK_SPEED_HZ: f64 = 1789773.;
pub const CYCLES_PER_FRAME: f64 = 29780.5;

bitflags::bitflags! {
    /// Flags for the cpu register
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        const CARRY = 1;
        const ZERO = 1 << 1;
        const INTERRUPT = 1 << 2;
        const DECIMAL = 1 << 3;
        // Set for pushing with BRK and PHP instructions will always be the same internally
        #[bitflags(flag_name = "")]
        const BREAK = 1 << 4;
        /// Always set
        #[bitflags(flag_name = "")]
        const UNUSED = 1 << 5;
        /// Set if arithmetic overflowed 8-bit signed number
        const OVERFLOW = 1 << 6;
        const NEGATIVE = 1 << 7;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::UNUSED
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum CpuError {
    #[error("CPU encountered a halt instruction")]
    Halted,
}

/// Emulated 6502 CPU
#[derive(Clone, Default)]
pub struct Cpu {
    /// Program counter
    pub pc: u16,
    // Stack pointer
    pub sp: u8,
    // Accumulator
    pub a: u8,
    // X register
    pub x: u8,
    // Y register
    pub y: u8,
    pub flags: Flags,
    pub bus: CpuBus,
    // Temp value storing the operand address
    operand_address: Option<u16>,
}

impl Cpu {
    /// Execute the next instruction at the pc
    /// Returns the number of cpu cycles that the instruction took to execute
    pub fn execute_next(&mut self) -> Result<u32, CpuError> {
        self.bus.cpu_cycles_since_inst = 0;

        let byte = self.read_u8_at_pc();
        if let Some(opcode) = Opcode::from_byte(byte) {
            self.operand_address = self.read_operand_address(opcode.addr_mode);
            self.execute(opcode)?;
        }

        if self.bus.require_nmi() {
            self.interrupt(0xfffa, Flags::empty(), 2);
        }

        Ok(self.bus.cpu_cycles_since_inst)
    }

    // fn irq(&mut self) {
    //     if !self.flags.contains(Flags::INTERRUPT) {
    //         self.interrupt(0xfffe, Flags::empty());
    //         self.bus.clock();
    //     }
    // }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = Flags::default() | Flags::INTERRUPT;

        self.bus.cpu_cycles_total = 0;
        self.bus.cpu_cycles_since_inst = 0;
        self.pc = self.bus.read_u16(0xfffc);
        self.sp = 0xfd;
        self.bus.ppu.registers.control = Default::default();
        for _ in 0..5 {
            self.bus.clock();
        }
    }

    fn read_u8_at_pc(&mut self) -> u8 {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.bus.read_u8(pc)
    }

    fn read_u16_at_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(2);
        self.bus.read_u16(pc)
    }

    fn address_add_offset(&mut self, address: u16, offset: u8, mode: AddrMode) -> u16 {
        let address_offset = address.wrapping_add(offset as u16);
        let force_dummy = matches!(
            mode,
            AddrMode::AbsoluteYForceDummy
                | AddrMode::AbsoluteXForceDummy
                | AddrMode::IndirectYForceDummy
        );
        // The CPU will first add the offset with the low byte of the address and try to read from there
        // Then it will check if the page has been crossed and read with the correct high byte if needs be
        if force_dummy || address_offset & 0xff00 != address & 0xff00 {
            // The dummy read if page crossed
            self.bus
                .read_u8((address & 0xff00) | (address_offset & 0x00ff));
        }
        address_offset
    }

    /// Returns the target address of the value based on the addressing mode and the operand
    fn read_operand_address(&mut self, mode: AddrMode) -> Option<u16> {
        Some(match mode {
            AddrMode::Accumulator => return None,
            AddrMode::Implied => return None,
            AddrMode::Immediate => {
                let address = self.pc;
                self.pc = self.pc.wrapping_add(1);
                address
            }
            AddrMode::ZeroPage => self.read_u8_at_pc() as u16,
            AddrMode::ZeroPageX => {
                self.bus.clock();
                self.read_u8_at_pc().wrapping_add(self.x) as u16
            }
            AddrMode::ZeroPageY => {
                self.bus.clock();
                self.read_u8_at_pc().wrapping_add(self.y) as u16
            }
            AddrMode::Absolute => self.read_u16_at_pc(),
            AddrMode::AbsoluteX | AddrMode::AbsoluteXForceDummy => {
                let address = self.read_u16_at_pc();
                self.address_add_offset(address, self.x, mode)
            }
            AddrMode::AbsoluteY | AddrMode::AbsoluteYForceDummy => {
                let address = self.read_u16_at_pc();
                self.address_add_offset(address, self.y, mode)
            }
            AddrMode::Indirect => {
                let indirect_address = self.read_u16_at_pc();
                self.bus.read_u16_wrapped(indirect_address)
            }
            AddrMode::IndirectX => {
                let indirect_address = self.read_u8_at_pc().wrapping_add(self.x);
                self.bus.clock();
                self.bus.read_u16_wrapped(indirect_address as u16)
            }
            AddrMode::IndirectY | AddrMode::IndirectYForceDummy => {
                let indirect_address = self.read_u8_at_pc();
                let address = self.bus.read_u16_wrapped(indirect_address as u16);
                self.address_add_offset(address, self.y, mode)
            }
            AddrMode::Relative => {
                let offset = self.read_u8_at_pc() as i8 as u16;
                self.pc.wrapping_add(offset)
            }
        })
    }

    fn read_operand_value(&mut self) -> Option<u8> {
        let value = self.bus.read_u8(self.operand_address?);
        self.set_zero_neg_flags(value);
        Some(value)
    }

    fn execute(&mut self, opcode: Opcode) -> Result<(), CpuError> {
        match opcode.name {
            // -- Stack --
            "pha" => self.stack_push(self.a),
            "php" => self.stack_push((self.flags | Flags::BREAK).bits()),
            "pla" => self.a = self.stack_pop(),
            "plp" => self.plp(),

            // -- Shift and rotate --
            "asl" => drop(self.shift(true, false)), // Use drop to return nothing
            "lsr" => drop(self.shift(false, false)),
            "rol" => drop(self.shift(true, true)),
            "ror" => drop(self.shift(false, true)),

            "slo" => self.a |= self.shift(true, false), // asl + ora
            "rla" => self.a &= self.shift(true, true),  // rol + and
            "sre" => self.a ^= self.shift(false, false), // lsr + eor
            "rra" => {
                // ror + adc
                let adder = self.shift(false, true);
                self.add_carry(adder);
            }

            // -- Arithmetic --
            "adc" => {
                let adder = self.read_operand_value().unwrap();
                self.add_carry(adder)
            }
            "sbc" => {
                // Inverting results in inverting the sign so the adc can be resued for sbc
                let adder = !self.read_operand_value().unwrap();
                self.add_carry(adder)
            }

            // -- Increment and decrement --
            "inc" => drop(self.increment(1, None)),
            "dec" => drop(self.increment(-1, None)),
            "inx" => self.x = self.increment(1, Some(self.x)),
            "iny" => self.y = self.increment(1, Some(self.y)),
            "dex" => self.x = self.increment(-1, Some(self.x)),
            "dey" => self.y = self.increment(-1, Some(self.y)),
            "isc" => {
                // inc + adc
                let adder = !self.increment(1, None);
                self.add_carry(adder);
            }
            "dcp" => {
                // dec + cmp
                let value = self.increment(-1, None);
                self.compare(self.a, Some(value));
            }

            // -- Register loads --
            "lda" => self.a = self.read_operand_value().unwrap(),
            "ldx" => self.x = self.read_operand_value().unwrap(),
            "ldy" => self.y = self.read_operand_value().unwrap(),
            "lax" => {
                self.a = self.read_operand_value().unwrap();
                self.x = self.a;
            }
            "las" => {
                self.a = self.read_operand_value().unwrap() & self.sp;
                (self.x, self.sp) = (self.a, self.a);
            }
            "lxa" => {
                self.a &= self.read_operand_value().unwrap();
                self.x = self.transfer(self.a);
            }

            // -- Register stores --
            "sta" => self.bus.write_u8(self.operand_address.unwrap(), self.a),
            "stx" => self.bus.write_u8(self.operand_address.unwrap(), self.x),
            "sty" => self.bus.write_u8(self.operand_address.unwrap(), self.y),
            "sax" => self
                .bus
                .write_u8(self.operand_address.unwrap(), self.a & self.x),

            // -- Register transfers --
            "tax" => self.x = self.transfer(self.a),
            "tay" => self.y = self.transfer(self.a),
            "tsx" => self.x = self.transfer(self.sp),
            "txa" => self.a = self.transfer(self.x),
            "txs" => {
                self.sp = self.x;
                self.bus.clock();
            }
            "tya" => self.a = self.transfer(self.y),

            // -- Flag clear and set --
            "clc" => self.flag(Flags::CARRY, false),
            "cld" => self.flag(Flags::DECIMAL, false),
            "cli" => self.flag(Flags::INTERRUPT, false),
            "clv" => self.flag(Flags::OVERFLOW, false),
            "sec" => self.flag(Flags::CARRY, true),
            "sed" => self.flag(Flags::DECIMAL, true),
            "sei" => self.flag(Flags::INTERRUPT, true),

            // -- Logic --
            "and" => self.a &= self.read_operand_value().unwrap(),
            "eor" => self.a ^= self.read_operand_value().unwrap(),
            "ora" => self.a |= self.read_operand_value().unwrap(),
            "bit" => self.bit(),
            "anc" => {
                self.a &= self.read_operand_value().unwrap();
                self.flags.set(Flags::CARRY, self.a & 0b1000_0000 != 0)
            }
            "asr" => {
                self.a &= self.read_operand_value().unwrap();
                self.a = self.calc_shift(self.a, false, false);
            }
            "arr" => self.arr(),

            "cmp" => self.compare(self.a, None),
            "cpx" => self.compare(self.x, None),
            "cpy" => self.compare(self.y, None),

            // -- Control flow --
            "jmp" => self.pc = self.operand_address.unwrap(),
            "jsr" => self.jsr(),
            "rts" => self.rts(),
            "brk" => self.interrupt(0xfffe, Flags::BREAK, 1),
            "rti" => self.rti(),

            "bcc" => self.branch(!self.flags.contains(Flags::CARRY)),
            "bcs" => self.branch(self.flags.contains(Flags::CARRY)),
            "beq" => self.branch(self.flags.contains(Flags::ZERO)),
            "bmi" => self.branch(self.flags.contains(Flags::NEGATIVE)),
            "bne" => self.branch(!self.flags.contains(Flags::ZERO)),
            "bpl" => self.branch(!self.flags.contains(Flags::NEGATIVE)),
            "bvc" => self.branch(!self.flags.contains(Flags::OVERFLOW)),
            "bvs" => self.branch(self.flags.contains(Flags::OVERFLOW)),

            // Does nothing
            "nop" => self.nop(),
            "hlt" => return Err(CpuError::Halted),
            _ => unreachable!("invalid opcode name {}", opcode.name),
        }

        match opcode.name {
            "las" | "pla" | "slo" | "rla" | "sre" | "ora" | "eor" | "and" | "anc" => {
                self.set_zero_neg_flags(self.a)
            }
            _ => (),
        }
        Ok(())
    }

    fn set_zero_neg_flags(&mut self, value: u8) {
        self.flags.set(Flags::ZERO, value == 0);
        self.flags.set(Flags::NEGATIVE, value & 0b1000_0000 != 0);
    }

    // This just returns the value but also clocks the bus and sets zero and neg flags for certain instructions
    fn transfer(&mut self, value: u8) -> u8 {
        self.bus.clock();
        self.set_zero_neg_flags(value);
        value
    }

    fn stack_push(&mut self, value: u8) {
        self.bus.write_u8(0x100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
        self.bus.clock();
    }

    fn stack_push_u16(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.bus
            .write_u16(0x100 + self.sp.wrapping_add(1) as u16, value);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.clock();
        self.bus.clock();
        self.bus.read_u8(0x100 + self.sp as u16)
    }

    fn stack_pop_u16(&mut self) -> u16 {
        self.sp = self.sp.wrapping_add(2);
        self.bus.read_u16(0x100 + self.sp.wrapping_sub(1) as u16)
    }

    fn plp(&mut self) {
        self.flags = Flags::from_bits(self.stack_pop()).unwrap();
        self.flags.insert(Flags::UNUSED);
        self.flags.remove(Flags::BREAK);
    }

    fn calc_shift(&mut self, value: u8, is_left: bool, contains_carry: bool) -> u8 {
        let carry = (self.flags.contains(Flags::CARRY) && contains_carry) as u8;
        let (result, carry_mask) = match is_left {
            true => ((value << 1) | carry, 0b1000_0000),
            false => ((value >> 1) | (carry << 7), 0b0000_0001),
        };

        self.flags.set(Flags::CARRY, value & carry_mask != 0);
        self.set_zero_neg_flags(result);
        result
    }

    fn shift(&mut self, is_left: bool, contains_carry: bool) -> u8 {
        let value = self.read_operand_value().unwrap_or(self.a);
        // Read-modify-update instructions like shift whatever immediately writes the value in the
        // same cycle as performing the operation
        if let Some(address) = self.operand_address {
            self.bus.write_u8(address, value);
        } else {
            // Still a cycle for implied addressing mode
            self.bus.clock();
        };

        let result = self.calc_shift(value, is_left, contains_carry);
        if let Some(address) = self.operand_address {
            self.bus.write_u8(address, result);
        } else {
            self.a = result;
        }
        result
    }

    fn increment(&mut self, sign: i8, value_override: Option<u8>) -> u8 {
        let value = value_override
            .or_else(|| self.read_operand_value())
            .unwrap();
        let result = (value as i8).wrapping_add(sign) as u8;
        if value_override.is_none() {
            // Should not be a implied addressing mode
            self.bus.write_u8(self.operand_address.unwrap(), value); // See shift func
            self.bus.write_u8(self.operand_address.unwrap(), result);
        } else {
            // Still a clock for the op when implied
            self.bus.clock();
        }
        self.set_zero_neg_flags(result);
        result
    }

    /// Set overflow if the resulting addition overflowed a (negative) 8-bit number with 2's compliment
    fn set_overflow_flag(&mut self, a: u8, adder: u8, result: u8) {
        let adder_same_sign = (a ^ adder) & 0b1000_0000 == 0;
        let result_changed_sign = (a ^ result) & 0b1000_0000 != 0;
        self.flags
            .set(Flags::OVERFLOW, adder_same_sign && result_changed_sign);
    }

    fn add_carry(&mut self, adder: u8) {
        let carry = self.flags.contains(Flags::CARRY);
        let result = adder as u16 + self.a as u16 + carry as u16;
        self.flags.set(Flags::CARRY, result > 0xff);

        self.set_overflow_flag(self.a, adder, result as u8);
        self.set_zero_neg_flags(result as u8);
        self.a = result as u8;
    }

    fn flag(&mut self, flag: Flags, value: bool) {
        self.flags.set(flag, value);
        self.bus.clock();
    }

    fn arr(&mut self) {
        self.a &= self.read_operand_value().unwrap();
        self.a = self.calc_shift(self.a, false, true);
        let bit_5 = (self.a & 0b0001_0000) >> 4;
        let bit_6 = (self.a & 0b0010_0000) >> 5;
        self.flags.set(Flags::CARRY, bit_5 == 1);
        self.flags.set(Flags::OVERFLOW, bit_5 ^ bit_6 == 1);
    }

    fn bit(&mut self) {
        let value = self.read_operand_value().unwrap();
        let result = self.a & value;
        self.flags.set(Flags::ZERO, result == 0);
        self.flags.set(Flags::OVERFLOW, value & (0b0100_0000) != 0);
        self.flags.set(Flags::NEGATIVE, value & (0b1000_0000) != 0);
    }

    fn compare(&mut self, register: u8, value: Option<u8>) {
        let value = value.or_else(|| self.read_operand_value()).unwrap();
        self.flags.set(Flags::CARRY, register >= value);
        self.set_zero_neg_flags(register.wrapping_sub(value));
    }

    fn jsr(&mut self) {
        self.stack_push_u16(self.pc - 1);
        self.bus.clock();
        self.pc = self.operand_address.unwrap();
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_u16() + 1;
        for _ in 0..3 {
            self.bus.clock();
        }
    }

    fn rti(&mut self) {
        self.plp();
        self.pc = self.stack_pop_u16();
    }

    fn interrupt(&mut self, load_vector: u16, push_flags: Flags, extra_clocks: usize) {
        self.stack_push_u16(self.pc);
        self.stack_push((self.flags | push_flags).bits());
        self.flags.set(Flags::INTERRUPT, true);
        self.pc = self.bus.read_u16(load_vector);
        for _ in 0..extra_clocks {
            self.bus.clock();
        }
    }

    fn branch(&mut self, condition: bool) {
        let address = self.operand_address.unwrap();
        if condition {
            self.bus.clock();
            if address & 0xff00 != self.pc & 0xff00 {
                self.bus.clock();
            }
            self.pc = address;
        }
    }

    fn nop(&mut self) {
        if let Some(address) = self.operand_address {
            // Dummy read for nop instructions with address
            self.bus.read_u8(address);
        } else {
            self.bus.clock();
        }
    }
}
