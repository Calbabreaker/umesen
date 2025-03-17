mod bus;
mod disassembler;
mod opcode;

use crate::CpuError;
use bus::CpuBus;
pub use disassembler::Disassembler;
pub use opcode::{AddrMode, Opcode};

/// Number of clock cycles per second
pub const CLOCK_SPEED_HZ: u32 = 1789773;
/// Number of cpu clock cycles per ppu frame actually meant to be 29780.5 but should be close enough
pub const CYCLES_PER_FRAME: u32 = 29780;

bitflags::bitflags! {
    /// Flags for the cpu register
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        const CARRY = 1;
        const ZERO = 1 << 1;
        const INTERRUPT = 1 << 2;
        /// Flag for binary coded decimal where hex 0x00->0x99 is decimal 0->99
        const DECIMAL = 1 << 3;
        // Set for pushing with BRK and PHP instructions
        const BREAK = 1 << 4;
        /// Always set
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

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flag_map = [
            (Flags::CARRY, "C"),
            (Flags::ZERO, "Z"),
            (Flags::INTERRUPT, "I"),
            (Flags::DECIMAL, "D"),
            (Flags::BREAK, "B"),
            (Flags::OVERFLOW, "O"),
            (Flags::NEGATIVE, "N"),
        ];
        for (flag, name) in flag_map {
            write!(f, "{} ", if self.contains(flag) { name } else { "-" })?;
        }
        Ok(())
    }
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
    operand_address: Option<u16>,
}

impl Cpu {
    /// Execute the next instruction at the pc
    /// Returns the number of cpu cycles to wait as the result of executing the instruction
    pub fn execute_next(&mut self) -> Result<u32, CpuError> {
        self.bus.cpu_cycles_to_wait = 0;

        if self.bus.require_nmi() {
            self.nmi();
        }

        let byte = self.read_u8_at_pc();
        let opcode = Opcode::from_byte(byte).ok_or(CpuError::UnknownOpcode(byte))?;
        self.operand_address = self.read_operand_address(opcode.addr_mode);
        self.execute(&opcode);

        Ok(self.bus.cpu_cycles_to_wait)
    }

    fn irq(&mut self) {
        if !self.flags.contains(Flags::INTERRUPT) {
            self.interrupt(0xfffe, Flags::empty());
            self.bus.clock();
        }
    }

    fn nmi(&mut self) {
        self.interrupt(0xfffa, Flags::empty());
        self.bus.clock();
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = Flags::default() | Flags::INTERRUPT;

        self.bus.cpu_cycles_total = 0;
        self.bus.cpu_cycles_to_wait = 0;
        self.pc = self.bus.read_u16(0xfffc);
        self.sp = 0xfd;
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
        let address_added = address.wrapping_add(offset as u16);
        let force_clock = matches!(
            mode,
            AddrMode::AbsoluteYForceClock
                | AddrMode::AbsoluteXForceClock
                | AddrMode::IndirectYForceClock
        );
        // Check page cross
        if force_clock || address & 0xff00 != address_added & 0xff00 {
            self.bus.clock();
        }
        address_added
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
            AddrMode::AbsoluteX | AddrMode::AbsoluteXForceClock => {
                let address = self.read_u16_at_pc();
                self.address_add_offset(address, self.x, mode)
            }
            AddrMode::AbsoluteY | AddrMode::AbsoluteYForceClock => {
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
            AddrMode::IndirectY | AddrMode::IndirectYForceClock => {
                let indirect_address = self.read_u8_at_pc();
                let address = self.bus.read_u16_wrapped(indirect_address as u16);
                self.address_add_offset(address, self.y, mode)
            }
            AddrMode::Relative => {
                // Create a twos complement of the offset then add to pc
                let offset = self.read_u8_at_pc() as i8 as u16;
                self.pc.wrapping_add(offset)
            }
        })
    }

    fn read_operand_value(&mut self) -> u8 {
        if let Some(address) = self.operand_address {
            self.bus.read_u8(address)
        } else {
            // Assume we're working with the accumulator for certain instructions
            self.a
        }
    }

    fn execute(&mut self, opcode: &Opcode) {
        match opcode.name {
            // -- Stack --
            "pha" => self.stack_push_clocked(self.a),
            "php" => self.php(),
            "pla" => self.a = self.stack_pop_clocked(),
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
                let adder = self.read_operand_value();
                self.add_carry(adder)
            }
            "sbc" => {
                // Inverting results in inverting the sign so the adc can be resued for sbc
                let adder = !self.read_operand_value();
                self.add_carry(adder)
            }

            // -- Increment and decrement --
            "inc" => drop(self.inc_mem(1)),
            "dec" => drop(self.inc_mem(-1)),
            "inx" => self.x = self.inc_val(self.x, 1),
            "iny" => self.y = self.inc_val(self.y, 1),
            "dex" => self.x = self.inc_val(self.x, -1),
            "dey" => self.y = self.inc_val(self.y, -1),

            "isc" => {
                let adder = !self.inc_mem(1);
                self.add_carry(adder);
            }
            "dcp" => {
                let value = self.inc_mem(-1);
                self.set_compare_flags(self.a, value);
            }

            // -- Register loads --
            "lda" => self.a = self.load_mem(),
            "ldx" => self.x = self.load_mem(),
            "ldy" => self.y = self.load_mem(),
            "lax" => {
                self.a = self.load_mem();
                self.x = self.a;
            }

            // -- Register stores --
            "sta" => self.store_mem(self.a),
            "stx" => self.store_mem(self.x),
            "sty" => self.store_mem(self.y),
            "sax" => self.store_mem(self.a & self.x),

            // -- Register transfers --
            "tax" => self.x = self.transfer(self.a),
            "tay" => self.y = self.transfer(self.a),
            "tsx" => self.x = self.transfer(self.sp),
            "txa" => self.a = self.transfer(self.x),
            "txs" => self.sp = self.x,
            "tya" => self.a = self.transfer(self.y),

            // -- Flag clear and set --
            "clc" => self.set_flag(Flags::CARRY, false),
            "cld" => self.set_flag(Flags::DECIMAL, false),
            "cli" => self.set_flag(Flags::INTERRUPT, false),
            "clv" => self.set_flag(Flags::OVERFLOW, false),
            "sec" => self.set_flag(Flags::CARRY, true),
            "sed" => self.set_flag(Flags::DECIMAL, true),
            "sei" => self.set_flag(Flags::INTERRUPT, true),

            // -- Logic --
            "and" => self.a &= self.load_mem(),
            "eor" => self.a ^= self.load_mem(),
            "ora" => self.a |= self.load_mem(),
            "bit" => self.bit(),

            "cmp" => self.compare(self.a),
            "cpx" => self.compare(self.x),
            "cpy" => self.compare(self.y),

            // -- Control flow --
            "jmp" => self.pc = self.operand_address.unwrap(),
            "jsr" => self.jsr(),
            "rts" => self.rts(),
            "brk" => self.interrupt(0xfffe, Flags::BREAK),
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
            "nop" => (),
            _ => unreachable!("invalid opcode name {}", opcode.name),
        };

        match opcode.name {
            "pla" | "and" | "eor" | "ora" | "slo" | "rla" | "sre" => {
                self.set_zero_neg_flags(self.a)
            }
            "nop" | "txs" => self.bus.clock(),
            _ => (),
        }
    }

    fn set_zero_neg_flags(&mut self, value: u8) {
        self.flags.set(Flags::ZERO, value == 0);
        self.flags.set(Flags::NEGATIVE, value & 0b1000_0000 != 0);
    }

    fn set_compare_flags(&mut self, register: u8, value: u8) {
        self.flags.set(Flags::CARRY, register >= value);
        self.set_zero_neg_flags(register.wrapping_sub(value));
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
    }

    fn stack_push_clocked(&mut self, value: u8) {
        self.stack_push(value);
        self.bus.clock();
    }

    fn stack_push_u16(&mut self, value: u16) {
        self.stack_push((value >> 8) as u8);
        self.stack_push(value as u8);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read_u8(0x100 + self.sp as u16)
    }

    fn stack_pop_clocked(&mut self) -> u8 {
        self.bus.clock();
        self.bus.clock();
        self.stack_pop()
    }

    fn stack_pop_u16(&mut self) -> u16 {
        (self.stack_pop() as u16) | ((self.stack_pop() as u16) << 8)
    }

    fn php(&mut self) {
        self.stack_push_clocked((self.flags | Flags::BREAK).bits());
    }

    fn plp(&mut self) {
        self.flags = Flags::from_bits(self.stack_pop_clocked()).unwrap();
        self.flags.insert(Flags::UNUSED);
        self.flags.remove(Flags::BREAK);
    }

    fn shift(&mut self, is_left: bool, contains_carry: bool) -> u8 {
        let value = self.read_operand_value();
        let carry = (self.flags.contains(Flags::CARRY) && contains_carry) as u8;
        let (result, carry_mask) = match is_left {
            true => ((value << 1) | carry, 0b1000_0000),
            false => ((value >> 1) | (carry << 7), 0b0000_0001),
        };

        self.flags.set(Flags::CARRY, value & carry_mask != 0);
        self.set_zero_neg_flags(result);
        self.bus.clock();

        if let Some(address) = self.operand_address {
            self.bus.write_u8(address, result);
        } else {
            self.a = result;
        }
        result
    }

    fn inc_val(&mut self, value: u8, sign: i8) -> u8 {
        let result = (value as i8).wrapping_add(sign) as u8;
        self.transfer(result)
    }

    fn inc_mem(&mut self, sign: i8) -> u8 {
        let value = self.read_operand_value();
        let result = self.inc_val(value, sign);
        self.store_mem(result);
        result
    }

    fn load_mem(&mut self) -> u8 {
        let value = self.read_operand_value();
        self.set_zero_neg_flags(value);
        value
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

    fn store_mem(&mut self, value: u8) {
        self.bus.write_u8(self.operand_address.unwrap(), value);
    }

    fn set_flag(&mut self, flag: Flags, value: bool) {
        self.flags.set(flag, value);
        self.bus.clock();
    }

    fn bit(&mut self) {
        let value = self.read_operand_value();
        let result = self.a & value;
        self.flags.set(Flags::ZERO, result == 0);
        self.flags.set(Flags::OVERFLOW, value & (0b0100_0000) != 0);
        self.flags.set(Flags::NEGATIVE, value & (0b1000_0000) != 0);
    }

    fn compare(&mut self, register: u8) {
        let value = self.read_operand_value();
        self.set_compare_flags(register, value);
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

    fn interrupt(&mut self, load_vector: u16, push_flags: Flags) {
        self.stack_push_u16(self.pc);
        self.stack_push((self.flags | push_flags).bits());
        self.flags.set(Flags::INTERRUPT, true);
        self.pc = self.bus.read_u16(load_vector);
        self.bus.clock();
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
}
