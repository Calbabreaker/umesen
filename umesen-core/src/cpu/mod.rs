#[cfg(test)]
mod test;

mod bus;

use std::{cell::RefCell, rc::Rc};

use crate::{CartridgeBoard, CpuError};
use bus::CpuBus;

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
    Relative,
}

#[derive(Default, Clone)]
pub struct Cpu {
    /// Program counter
    pc: u16,
    // Stack pointer
    sp: u8,
    // Accumulator
    a: u8,
    // X register
    x: u8,
    // Y register
    y: u8,
    flags: Flags,
    pub bus: CpuBus,
}

impl Cpu {
    pub fn execute_next(&mut self) -> Result<(), CpuError> {
        let opcode = self.read_byte_at_pc();
        self.execute(opcode)
    }

    pub fn irq(&mut self) {
        if !self.flags.contains(Flags::INTERRUPT) {
            self.interrupt(0xfffe);
        }
    }

    pub fn nmi(&mut self) {
        self.interrupt(0xfffa);
    }

    pub fn reset(&mut self) {
        self.pc = self.bus.read_word(0xfffc);
        for _ in 0..5 {
            self.bus.clock();
        }
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
        // Check page cross
        if force_clock || address & 0xff00 != address_added & 0xff00 {
            self.bus.clock();
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
                    msb << 8 | lsb
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
        }
    }

    /// Returns (value, address)
    fn read_operand(&mut self, mode: AddrMode) -> (u8, u16) {
        let address = self.read_operand_address(mode);
        (self.bus.read_byte(address), address)
    }

    fn execute(&mut self, opcode: u8) -> Result<(), CpuError> {
        use AddrMode::*;
        match opcode {
            // -- Stack --
            0x48 => self.stack_push(self.a),            // pha
            0x08 => self.stack_push(self.flags.bits()), // php
            0x68 => self.pla(),
            0x28 => self.plp(),

            // -- Shift and rotate --
            // asl
            0x0a => self.a = self.shift(self.a, '<', false),
            0x06 => self.shift_mem('<', false, ZeroPage),
            0x16 => self.shift_mem('<', false, ZeroPageX),
            0x0e => self.shift_mem('<', false, Absolute),
            0x1e => self.shift_mem('<', false, AbsoluteXForceClock),

            // lsr
            0x4a => self.a = self.shift(self.a, '>', false),
            0x46 => self.shift_mem('>', false, ZeroPage),
            0x56 => self.shift_mem('>', false, ZeroPageX),
            0x4e => self.shift_mem('>', false, Absolute),
            0x5e => self.shift_mem('>', false, AbsoluteXForceClock),

            // rol
            0x2a => self.a = self.shift(self.a, '<', true),
            0x26 => self.shift_mem('<', true, ZeroPage),
            0x36 => self.shift_mem('<', true, ZeroPageX),
            0x2e => self.shift_mem('<', true, Absolute),
            0x3e => self.shift_mem('<', true, AbsoluteXForceClock),

            // ror
            0x6a => self.a = self.shift(self.a, '>', true),
            0x66 => self.shift_mem('>', true, ZeroPage),
            0x76 => self.shift_mem('>', true, ZeroPageX),
            0x6e => self.shift_mem('>', true, Absolute),
            0x7e => self.shift_mem('>', true, AbsoluteXForceClock),

            // -- Arithmetic --
            0x69 => self.adc(Immediate),
            0x65 => self.adc(ZeroPage),
            0x75 => self.adc(ZeroPageX),
            0x6d => self.adc(Absolute),
            0x7d => self.adc(AbsoluteX),
            0x79 => self.adc(AbsoluteY),
            0x61 => self.adc(IndirectX),
            0x71 => self.adc(IndirectY),

            0xe9 => self.sbc(Immediate),
            0xe5 => self.sbc(ZeroPage),
            0xf5 => self.sbc(ZeroPageX),
            0xed => self.sbc(Absolute),
            0xfd => self.sbc(AbsoluteX),
            0xf9 => self.sbc(AbsoluteY),
            0xe1 => self.sbc(IndirectX),
            0xf1 => self.sbc(IndirectY),

            // -- Increment and decrement --
            // inc
            0xe6 => self.inc_mem(1, ZeroPage),
            0xf6 => self.inc_mem(1, ZeroPageX),
            0xee => self.inc_mem(1, Absolute),
            0xfe => self.inc_mem(1, AbsoluteXForceClock),

            // dec
            0xc6 => self.inc_mem(-1, ZeroPage),
            0xd6 => self.inc_mem(-1, ZeroPageX),
            0xce => self.inc_mem(-1, Absolute),
            0xde => self.inc_mem(-1, AbsoluteXForceClock),

            0xe8 => self.x = self.inc_val(1, self.x), // inx
            0xc8 => self.y = self.inc_val(1, self.y), // iny
            0xca => self.x = self.inc_val(-1, self.x), // dex
            0x88 => self.y = self.inc_val(-1, self.y), // dey

            // -- Register loads --
            // lda
            0xa9 => self.a = self.load_mem(Immediate),
            0xa5 => self.a = self.load_mem(ZeroPage),
            0xb5 => self.a = self.load_mem(ZeroPageX),
            0xad => self.a = self.load_mem(Absolute),
            0xbd => self.a = self.load_mem(AbsoluteX),
            0xb9 => self.a = self.load_mem(AbsoluteY),
            0xa1 => self.a = self.load_mem(IndirectX),
            0xb1 => self.a = self.load_mem(IndirectY),

            // ldx
            0xa2 => self.x = self.load_mem(Immediate),
            0xa6 => self.x = self.load_mem(ZeroPage),
            0xb6 => self.x = self.load_mem(ZeroPageY),
            0xae => self.x = self.load_mem(Absolute),
            0xbe => self.x = self.load_mem(AbsoluteY),

            // ldy
            0xa0 => self.y = self.load_mem(Immediate),
            0xa4 => self.y = self.load_mem(ZeroPage),
            0xb4 => self.y = self.load_mem(ZeroPageX),
            0xac => self.y = self.load_mem(Absolute),
            0xbc => self.y = self.load_mem(AbsoluteX),

            // -- Register stores --
            // sta
            0x85 => self.store_mem(self.a, ZeroPage),
            0x95 => self.store_mem(self.a, ZeroPageX),
            0x8d => self.store_mem(self.a, Absolute),
            0x9d => self.store_mem(self.a, AbsoluteXForceClock),
            0x99 => self.store_mem(self.a, AbsoluteYForceClock),
            0x81 => self.store_mem(self.a, IndirectX),
            0x91 => self.store_mem(self.a, IndirectYForceClock),

            // stx
            0x8e => self.store_mem(self.x, Absolute),
            0x86 => self.store_mem(self.x, ZeroPage),
            0x96 => self.store_mem(self.x, ZeroPageY),

            // sty
            0x8c => self.store_mem(self.y, Absolute),
            0x84 => self.store_mem(self.y, ZeroPage),
            0x94 => self.store_mem(self.y, ZeroPageX),

            // -- Register transfers --
            0xaa => self.x = self.copy_val(self.a),  // tax
            0xa8 => self.y = self.copy_val(self.a),  // tay
            0xba => self.x = self.copy_val(self.sp), // tsx
            0x8a => self.a = self.copy_val(self.x),  // txa
            0x9a => self.sp = self.copy_val(self.x), // txs
            0x98 => self.a = self.copy_val(self.y),  // tya

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
            0x29 => self.a &= self.load_mem(Immediate),
            0x25 => self.a &= self.load_mem(ZeroPage),
            0x35 => self.a &= self.load_mem(ZeroPageX),
            0x2d => self.a &= self.load_mem(Absolute),
            0x3d => self.a &= self.load_mem(AbsoluteX),
            0x39 => self.a &= self.load_mem(AbsoluteY),
            0x21 => self.a &= self.load_mem(IndirectX),
            0x31 => self.a &= self.load_mem(IndirectY),

            0x2c => self.bit(Absolute),
            0x24 => self.bit(ZeroPage),

            // eor
            0x49 => self.a ^= self.load_mem(Immediate),
            0x45 => self.a ^= self.load_mem(ZeroPage),
            0x55 => self.a ^= self.load_mem(ZeroPageX),
            0x4d => self.a ^= self.load_mem(Absolute),
            0x5d => self.a ^= self.load_mem(AbsoluteX),
            0x59 => self.a ^= self.load_mem(AbsoluteY),
            0x41 => self.a ^= self.load_mem(IndirectX),
            0x51 => self.a ^= self.load_mem(IndirectY),

            // ora
            0x09 => self.a |= self.load_mem(Immediate),
            0x05 => self.a |= self.load_mem(ZeroPage),
            0x15 => self.a |= self.load_mem(ZeroPageX),
            0x0d => self.a |= self.load_mem(Absolute),
            0x1d => self.a |= self.load_mem(AbsoluteX),
            0x19 => self.a |= self.load_mem(AbsoluteY),
            0x01 => self.a |= self.load_mem(IndirectX),
            0x11 => self.a |= self.load_mem(IndirectY),

            // cmp
            0xc9 => self.compare(self.a, Immediate),
            0xc5 => self.compare(self.a, ZeroPage),
            0xd5 => self.compare(self.a, ZeroPageX),
            0xcd => self.compare(self.a, Absolute),
            0xdd => self.compare(self.a, AbsoluteX),
            0xd9 => self.compare(self.a, AbsoluteY),
            0xc1 => self.compare(self.a, IndirectX),
            0xd1 => self.compare(self.a, IndirectY),

            // cpx
            0xe0 => self.compare(self.x, Immediate),
            0xe4 => self.compare(self.x, ZeroPage),
            0xec => self.compare(self.x, Absolute),

            // cpy
            0xc0 => self.compare(self.y, Immediate),
            0xc4 => self.compare(self.y, ZeroPage),
            0xcc => self.compare(self.y, Absolute),

            // -- Control flow --
            0x4c => self.jmp(Absolute),
            0x6c => self.jmp(Indirect),
            0x20 => self.jsr(),
            0x60 => self.rts(),
            0x00 => self.brk(),
            0x40 => self.rti(),

            0x90 => self.branch(!self.flags.contains(Flags::CARRY)), // bcc
            0xb0 => self.branch(self.flags.contains(Flags::CARRY)),  // bcs
            0xf0 => self.branch(self.flags.contains(Flags::ZERO)),   // beq
            0x30 => self.branch(self.flags.contains(Flags::NEGATIVE)), // bmi
            0xd0 => self.branch(!self.flags.contains(Flags::ZERO)),  // bne
            0x10 => self.branch(!self.flags.contains(Flags::NEGATIVE)), // bpl
            0x50 => self.branch(!self.flags.contains(Flags::OVERFLOW)), // bvc
            0x70 => self.branch(self.flags.contains(Flags::OVERFLOW)), // bvs

            // Does nothing nop
            0xea => self.bus.clock(),

            _ => return Err(CpuError::UnknownOpcode(opcode)),
        };
        Ok(())
    }

    fn set_zero_neg_flags(&mut self, value: u8) {
        self.flags.set(Flags::ZERO, value == 0);
        self.flags.set(Flags::NEGATIVE, value & 0b1000_0000 != 0);
    }

    fn copy_val(&mut self, value: u8) -> u8 {
        self.bus.clock();
        self.set_zero_neg_flags(value);
        value
    }

    fn unclocked_stack_push(&mut self, value: u8) {
        self.bus.write_byte(0x100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_push(&mut self, value: u8) {
        self.unclocked_stack_push(value);
        self.bus.clock();
    }

    fn stack_push_word(&mut self, value: u16) {
        self.unclocked_stack_push((value >> 8) as u8);
        self.stack_push(value as u8);
    }

    fn unclocked_stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read_byte(0x100 + self.sp as u16)
    }

    fn stack_pop(&mut self) -> u8 {
        self.bus.clock();
        self.bus.clock();
        self.unclocked_stack_pop()
    }

    fn stack_pop_word(&mut self) -> u16 {
        self.unclocked_stack_pop() as u16 | (self.stack_pop() as u16) << 8
    }

    fn pla(&mut self) {
        self.a = self.stack_pop();
        self.set_zero_neg_flags(self.a);
    }

    fn plp(&mut self) {
        self.flags = Flags::from_bits(self.stack_pop()).unwrap();
    }

    fn shift(&mut self, value: u8, dir: char, contains_carry: bool) -> u8 {
        let carry = (self.flags.contains(Flags::CARRY) && contains_carry) as u8;
        let (result, carry_mask) = match dir {
            '<' => ((value << 1) | carry, 0b1000_0000),
            '>' => ((value >> 1) | (carry << 7), 0b0000_0001),
            _ => unreachable!(),
        };
        self.flags.set(Flags::CARRY, value & carry_mask != 0);
        self.copy_val(result)
    }

    fn shift_mem(&mut self, dir: char, contains_carry: bool, mode: AddrMode) {
        let (value, address) = self.read_operand(mode);
        let result = self.shift(value, dir, contains_carry);
        self.bus.write_byte(address, result);
    }

    fn inc_val(&mut self, sign: i8, value: u8) -> u8 {
        let result = (value as i8).wrapping_add(sign) as u8;
        self.copy_val(result)
    }

    fn inc_mem(&mut self, sign: i8, mode: AddrMode) {
        let (value, address) = self.read_operand(mode);
        let result = self.inc_val(sign, value);
        self.bus.write_byte(address, result);
    }

    fn load_mem(&mut self, mode: AddrMode) -> u8 {
        let value = self.read_operand(mode).0;
        self.set_zero_neg_flags(value);
        value
    }

    // Set overflow if the resulting addition overflowed a 8-bit number with 2's compliment
    fn set_overflow_flag(&mut self, a: u8, adder: u8, result: u8) {
        let adder_same_sign = (a ^ adder) & 0b1000_0000 == 0;
        let result_changed_sign = (a ^ result) & 0b1000_0000 != 0;
        self.flags
            .set(Flags::OVERFLOW, adder_same_sign && result_changed_sign);
    }

    fn add_carry(&mut self, adder: u8, carry: bool) {
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
        let (adder, _) = self.read_operand(mode);
        self.add_carry(adder, self.flags.contains(Flags::CARRY));
    }

    fn sbc(&mut self, mode: AddrMode) {
        let (adder, _) = self.read_operand(mode);
        // Inverting results in inverting the sign so the adc can be resued
        self.add_carry(!adder, !self.flags.contains(Flags::CARRY));
    }

    fn store_mem(&mut self, register: u8, mode: AddrMode) {
        let address = self.read_operand_address(mode);
        self.bus.write_byte(address, register);
    }

    fn set_flag(&mut self, flag: Flags, value: bool) {
        self.flags.set(flag, value);
        self.bus.clock();
    }

    fn bit(&mut self, mode: AddrMode) {
        let (value, _) = self.read_operand(mode);
        let result = self.a & value;
        self.flags.set(Flags::ZERO, result == 0);
        self.flags.set(Flags::OVERFLOW, value & (0b0100_0000) != 0);
        self.flags.set(Flags::NEGATIVE, value & (0b1000_0000) != 0);
    }

    fn compare(&mut self, register: u8, mode: AddrMode) {
        let (value, _) = self.read_operand(mode);
        let result = register.wrapping_sub(value);
        self.flags.set(Flags::CARRY, register >= value);
        self.set_zero_neg_flags(result);
    }

    fn jmp(&mut self, mode: AddrMode) {
        let address = self.read_operand_address(mode);
        self.pc = address;
    }

    fn jsr(&mut self) {
        let address = self.read_operand_address(AddrMode::Absolute);
        self.stack_push_word(self.pc - 1);
        self.pc = address;
    }

    fn rts(&mut self) {
        self.pc = self.stack_pop_word() + 1;
        self.bus.clock();
    }

    fn rti(&mut self) {
        self.flags = Flags::from_bits(self.unclocked_stack_pop()).unwrap();
        self.pc = self.stack_pop_word();
    }

    fn brk(&mut self) {
        self.interrupt(0xfffe);
        self.flags.set(Flags::BREAK, true);
    }

    fn interrupt(&mut self, load_vector: u16) {
        self.stack_push_word(self.pc);
        self.unclocked_stack_push(self.flags.bits());
        self.flags.set(Flags::INTERRUPT, true);
        self.pc = self.bus.read_word(load_vector);
    }

    fn branch(&mut self, condition: bool) {
        let address = self.read_operand_address(AddrMode::Relative);
        if condition {
            self.bus.clock();
            if address & 0xff00 != self.pc & 0xff00 {
                self.bus.clock();
                self.bus.clock();
            }
            self.pc = address;
        }
    }
}
