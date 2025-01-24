#[cfg(test)]
mod test;

mod bus;
mod disassembler;
mod opcode;

use crate::CpuError;
use bus::CpuBus;
pub use disassembler::Disassembler;
pub use opcode::{AddrMode, Opcode};

bitflags::bitflags! {
    /// Flags for the cpu register
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        const CARRY = 1;
        const ZERO = 1 << 1;
        const INTERRUPT = 1 << 2;
        /// Flag for binary coded decimal where hex 0x00->0x99 is decimal 0->99
        const DECIMAL = 1 << 3;
        const BREAK = 1 << 4;
        // UNUSED should always be set but not internally for easier testing
        const UNUSED = 1 << 5;
        /// Set if arithmetic overflowed 8-bit signed number
        const OVERFLOW = 1 << 6;
        const NEGATIVE = 1 << 7;
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
#[derive(Default)]
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
    pub fn execute_next(&mut self) -> Result<(), CpuError> {
        if self.bus.require_nmi() {
            self.nmi();
        }

        let opcode = Opcode::from_byte(self.read_byte_at_pc())?;
        self.execute(opcode);
        Ok(())
    }

    fn irq(&mut self) {
        if !self.flags.contains(Flags::INTERRUPT) {
            self.interrupt(0xfffe);
            self.bus.clock();
        }
    }

    fn nmi(&mut self) {
        self.interrupt(0xfffa);
        self.bus.clock();
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = Flags::empty();

        self.bus.cpu_cycles = 0;
        self.pc = self.bus.read_word(0xfffc);
        self.sp = 0xff;
        for _ in 0..5 {
            self.bus.clock();
        }
    }

    fn read_byte_at_pc(&mut self) -> u8 {
        self.pc = self.pc.wrapping_add(1);
        self.bus.read_byte(self.pc - 1)
    }

    fn read_word_at_pc(&mut self) -> u16 {
        self.pc = self.pc.wrapping_add(2);
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
            AddrMode::ZeroPage => self.read_byte_at_pc() as u16,
            AddrMode::ZeroPageX => {
                self.bus.clock();
                self.read_byte_at_pc().wrapping_add(self.x) as u16
            }
            AddrMode::ZeroPageY => {
                self.bus.clock();
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
                let indirect_address = self.read_word_at_pc();
                // Emulate cross page read bug on original 6502
                if indirect_address & 0x00ff == 0x00ff {
                    let lsb = self.bus.read_byte(indirect_address) as u16;
                    // Get MSB from start of page
                    let msb = self.bus.read_byte(indirect_address & 0xff00) as u16;
                    (msb << 8) | lsb
                } else {
                    self.bus.read_word(indirect_address)
                }
            }
            AddrMode::IndirectX => {
                let indirect_address = self.read_byte_at_pc().wrapping_add(self.x);
                self.bus.clock();
                self.bus.read_word(indirect_address as u16)
            }
            AddrMode::IndirectY | AddrMode::IndirectYForceClock => {
                let indirect_address = self.read_byte_at_pc();
                let address = self.bus.read_word(indirect_address as u16);
                self.address_add_offset(address, self.y, mode)
            }
            AddrMode::Relative => {
                let offset = self.read_byte_at_pc() as i8 as i16;
                (self.pc as i16).wrapping_add(offset) as u16
            }
        })
    }

    fn read_operand_value(&mut self) -> u8 {
        if let Some(address) = self.operand_address {
            self.bus.read_byte(address)
        } else {
            // Assume we're working with the accumulator for certain instructions
            self.a
        }
    }

    fn execute(&mut self, opcode: Opcode) {
        self.operand_address = self.read_operand_address(opcode.addr_mode);
        match opcode.name {
            // -- Stack --
            "pha" => self.pha(),
            "php" => self.php(),
            "pla" => self.pla(),
            "plp" => self.plp(),

            // -- Shift and rotate --
            "asl" => self.shift('<', false),
            "lsr" => self.shift('>', false),
            "rol" => self.shift('<', true),
            "ror" => self.shift('>', true),

            // -- Arithmetic --
            "adc" => self.adc(),
            "sbc" => self.sbc(),

            // -- Increment and decrement --
            "inc" => self.inc_mem(1),
            "dec" => self.inc_mem(-1),
            "inx" => self.x = self.inc_val(1, self.x),
            "iny" => self.y = self.inc_val(1, self.y),
            "dex" => self.x = self.inc_val(-1, self.x),
            "dey" => self.y = self.inc_val(-1, self.y),

            // -- Register loads --
            "lda" => self.a = self.load_mem(),
            "ldx" => self.x = self.load_mem(),
            "ldy" => self.y = self.load_mem(),

            // -- Register stores --
            "sta" => self.store_mem(self.a),
            "stx" => self.store_mem(self.x),
            "sty" => self.store_mem(self.y),

            // -- Register transfers --
            "tax" => self.x = self.transfer(self.a),
            "tay" => self.y = self.transfer(self.a),
            "tsx" => self.x = self.transfer(self.sp),
            "txa" => self.a = self.transfer(self.x),
            "txs" => self.sp = self.transfer(self.x),
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
            "bit" => self.bit(),
            "eor" => self.a ^= self.load_mem(),
            "ora" => self.a |= self.load_mem(),
            "cmp" => self.compare(self.a),
            "cpx" => self.compare(self.x),
            "cpy" => self.compare(self.y),

            // -- Control flow --
            "jmp" => self.jmp(),
            "jsr" => self.jsr(),
            "rts" => self.rts(),
            "brk" => self.brk(),
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
            "nop" => self.bus.clock(),
            _ => unreachable!("invalid opcode name {}", opcode.name),
        }
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
        self.bus.write_byte(0x100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_push_word(&mut self, value: u16) {
        self.stack_push((value >> 8) as u8);
        self.stack_push(value as u8);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read_byte(0x100 + self.sp as u16)
    }

    fn stack_pop_word(&mut self) -> u16 {
        (self.stack_pop() as u16) | ((self.stack_pop() as u16) << 8)
    }

    fn pha(&mut self) {
        self.stack_push(self.a);
        self.bus.clock();
    }

    fn php(&mut self) {
        let flags = self.flags | Flags::UNUSED | Flags::BREAK;
        self.stack_push(flags.bits());
        self.bus.clock();
    }

    fn pla(&mut self) {
        self.a = self.stack_pop();
        self.set_zero_neg_flags(self.a);
        self.bus.clock();
        self.bus.clock();
    }

    fn plp(&mut self) {
        self.flags = Flags::from_bits(self.stack_pop()).unwrap();
        self.flags.remove(Flags::BREAK);
        self.flags.remove(Flags::UNUSED);
        self.bus.clock();
        self.bus.clock();
    }

    fn shift(&mut self, dir: char, contains_carry: bool) {
        let value = self.read_operand_value();
        let carry = (self.flags.contains(Flags::CARRY) && contains_carry) as u8;
        let (result, carry_mask) = match dir {
            '<' => ((value << 1) | carry, 0b1000_0000),
            '>' => ((value >> 1) | (carry << 7), 0b0000_0001),
            _ => unreachable!(),
        };

        self.flags.set(Flags::CARRY, result & carry_mask != 0);
        self.set_zero_neg_flags(result);
        self.bus.clock();

        if let Some(address) = self.operand_address {
            self.bus.write_byte(address, result);
        } else {
            self.a = result;
        }
    }

    fn inc_val(&mut self, sign: i8, value: u8) -> u8 {
        let result = (value as i8).wrapping_add(sign) as u8;
        self.transfer(result)
    }

    fn inc_mem(&mut self, sign: i8) {
        let value = self.read_operand_value();
        let result = self.inc_val(sign, value);
        self.bus.write_byte(self.operand_address.unwrap(), result);
    }

    fn load_mem(&mut self) -> u8 {
        let value = self.read_operand_value();
        self.set_zero_neg_flags(value);
        value
    }

    // Set overflow if the resulting addition overflowed a (negative) 8-bit number with 2's compliment
    fn set_overflow_flag(&mut self, a: u8, adder: u8, result: u8) {
        let adder_same_sign = (a ^ adder) & 0b1000_0000 == 0;
        let result_changed_sign = (a ^ result) & 0b1000_0000 != 0;
        self.flags
            .set(Flags::OVERFLOW, adder_same_sign && result_changed_sign);
    }

    fn add_carry(&mut self, adder: u8, carry: bool) {
        let mut result_word = adder as u16 + self.a as u16 + carry as u16;

        // Convert result back into binary if decimal enabled
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

    fn adc(&mut self) {
        let adder = self.read_operand_value();
        self.add_carry(adder, self.flags.contains(Flags::CARRY));
    }

    fn sbc(&mut self) {
        let adder = self.read_operand_value();
        // Inverting results in inverting the sign so the adc can be resued
        self.add_carry(!adder, !self.flags.contains(Flags::CARRY));
    }

    fn store_mem(&mut self, register: u8) {
        self.bus.write_byte(self.operand_address.unwrap(), register);
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
        let result = register.wrapping_sub(value);
        self.flags.set(Flags::CARRY, register >= value);
        self.set_zero_neg_flags(result);
    }

    fn jmp(&mut self) {
        self.pc = self.operand_address.unwrap();
    }

    fn jsr(&mut self) {
        self.stack_push_word(self.pc - 1);
        self.bus.clock();
        self.pc = self.operand_address.unwrap();
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_word() + 1;
        for _ in 0..3 {
            self.bus.clock();
        }
    }

    fn rti(&mut self) {
        self.plp();
        self.pc = self.stack_pop_word();
    }

    fn brk(&mut self) {
        self.flags.set(Flags::BREAK, true);
        self.interrupt(0xfffe);
        self.flags.set(Flags::BREAK, false);
    }

    fn interrupt(&mut self, load_vector: u16) {
        self.stack_push_word(self.pc);
        self.stack_push((self.flags | Flags::UNUSED).bits());
        self.flags.set(Flags::INTERRUPT, true);
        self.pc = self.bus.read_word(load_vector);
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
