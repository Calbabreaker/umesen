mod bus;
mod disassembler;
mod opcode;

use bus::CpuBus;
pub use disassembler::Disassembler;
pub use opcode::{AddrMode, Inst, Opcode};

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum InterruptKind {
    Brk,
    Irq,
    Nmi,
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
    // Temp value storing the operand address
    operand_address: Option<u16>,
}

impl Cpu {
    /// Execute the next instruction at the pc
    /// Returns the number of cpu cycles that the instruction took to execute
    pub fn execute_next(&mut self) -> Result<u32, CpuError> {
        self.bus.cpu_cycles_since_inst = 0;

        if self.bus.require_nmi() {
            self.interrupt(InterruptKind::Nmi);
        } else if self.bus.irq_status() && !self.flags.contains(Flags::INTERRUPT) {
            self.interrupt(InterruptKind::Irq);
        }

        let byte = self.read_at_pc();
        if let Some(opcode) = Opcode::from_byte(byte) {
            self.operand_address = self.read_operand_address(opcode.addr_mode);
            self.execute(opcode)?;
        }

        Ok(self.bus.cpu_cycles_since_inst)
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.flags = Flags::default() | Flags::INTERRUPT;

        self.bus.cpu_cycles_total = 0;
        self.bus.cpu_cycles_since_inst = 0;
        self.pc = self.bus.read_u16(0xfffc);
        self.sp = 0xfd;
        // Some roms freeze when soft loading if nmi is enabled for some reason
        self.bus.ppu.registers.control = Default::default();
        if let Some(c) = self.bus.cartridge.as_mut() {
            c.borrow_mut().reset()
        }

        for _ in 0..5 {
            self.bus.clock();
        }
    }

    fn read_at_pc(&mut self) -> u8 {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.bus.read(pc)
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
                .read((address & 0xff00) | (address_offset & 0x00ff));
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
            AddrMode::ZeroPage => self.read_at_pc() as u16,
            AddrMode::ZeroPageX => {
                self.bus.clock();
                self.read_at_pc().wrapping_add(self.x) as u16
            }
            AddrMode::ZeroPageY => {
                self.bus.clock();
                self.read_at_pc().wrapping_add(self.y) as u16
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
                let indirect_address = self.read_at_pc().wrapping_add(self.x);
                self.bus.clock();
                self.bus.read_u16_wrapped(indirect_address as u16)
            }
            AddrMode::IndirectY | AddrMode::IndirectYForceDummy => {
                let indirect_address = self.read_at_pc();
                let address = self.bus.read_u16_wrapped(indirect_address as u16);
                self.address_add_offset(address, self.y, mode)
            }
            AddrMode::Relative => {
                let offset = self.read_at_pc() as i8 as u16;
                self.pc.wrapping_add(offset)
            }
        })
    }

    fn read_operand_value(&mut self) -> Option<u8> {
        let value = self.bus.read(self.operand_address?);
        self.set_zero_neg_flags(value);
        Some(value)
    }

    fn execute(&mut self, opcode: Opcode) -> Result<(), CpuError> {
        match opcode.instruction {
            // -- Stack --
            Inst::Pha => self.stack_push(self.a),
            Inst::Php => self.stack_push((self.flags | Flags::BREAK).bits()),
            Inst::Pla => self.a = self.stack_pop(),
            Inst::Plp => self.plp(),

            // -- Shift and rotate --
            Inst::Asl => drop(self.shift(true, false)), // Use drop to return nothing
            Inst::Lsr => drop(self.shift(false, false)),
            Inst::Rol => drop(self.shift(true, true)),
            Inst::Ror => drop(self.shift(false, true)),

            Inst::Slo => self.a |= self.shift(true, false), // asl + ora
            Inst::Rla => self.a &= self.shift(true, true),  // rol + and
            Inst::Sre => self.a ^= self.shift(false, false), // lsr + eor
            Inst::Rra => {
                // ror + adc
                let adder = self.shift(false, true);
                self.add_carry(adder);
            }

            // -- Arithmetic --
            Inst::Adc => {
                let adder = self.read_operand_value().unwrap();
                self.add_carry(adder)
            }
            Inst::Sbc => {
                // Inverting results in inverting the sign so the adc can be resued for sbc
                let adder = !self.read_operand_value().unwrap();
                self.add_carry(adder)
            }

            // -- Increment and decrement --
            Inst::Inc => drop(self.increment(1, None)),
            Inst::Dec => drop(self.increment(-1, None)),
            Inst::Inx => self.x = self.increment(1, Some(self.x)),
            Inst::Iny => self.y = self.increment(1, Some(self.y)),
            Inst::Dex => self.x = self.increment(-1, Some(self.x)),
            Inst::Dey => self.y = self.increment(-1, Some(self.y)),
            Inst::Isc => {
                // inc + adc
                let adder = !self.increment(1, None);
                self.add_carry(adder);
            }
            Inst::Dcp => {
                // dec + cmp
                let value = self.increment(-1, None);
                self.compare(self.a, Some(value));
            }

            // -- Register loads --
            Inst::Lda => self.a = self.read_operand_value().unwrap(),
            Inst::Ldx => self.x = self.read_operand_value().unwrap(),
            Inst::Ldy => self.y = self.read_operand_value().unwrap(),
            Inst::Lax => {
                self.a = self.read_operand_value().unwrap();
                self.x = self.a;
            }
            Inst::Las => {
                self.a = self.read_operand_value().unwrap() & self.sp;
                (self.x, self.sp) = (self.a, self.a);
            }
            Inst::Lxa => {
                self.a &= self.read_operand_value().unwrap();
                self.x = self.transfer(self.a);
            }

            // -- Register stores --
            Inst::Sta => self.bus.write(self.operand_address.unwrap(), self.a),
            Inst::Stx => self.bus.write(self.operand_address.unwrap(), self.x),
            Inst::Sty => self.bus.write(self.operand_address.unwrap(), self.y),
            Inst::Sax => self
                .bus
                .write(self.operand_address.unwrap(), self.a & self.x),

            // -- Register transfers --
            Inst::Tax => self.x = self.transfer(self.a),
            Inst::Tay => self.y = self.transfer(self.a),
            Inst::Tsx => self.x = self.transfer(self.sp),
            Inst::Txa => self.a = self.transfer(self.x),
            Inst::Txs => {
                self.sp = self.x;
                self.bus.clock();
            }
            Inst::Tya => self.a = self.transfer(self.y),

            // -- Flag clear and set --
            Inst::Clc => self.flag(Flags::CARRY, false),
            Inst::Cld => self.flag(Flags::DECIMAL, false),
            Inst::Cli => self.flag(Flags::INTERRUPT, false),
            Inst::Clv => self.flag(Flags::OVERFLOW, false),
            Inst::Sec => self.flag(Flags::CARRY, true),
            Inst::Sed => self.flag(Flags::DECIMAL, true),
            Inst::Sei => self.flag(Flags::INTERRUPT, true),

            // -- Logic --
            Inst::And => self.a &= self.read_operand_value().unwrap(),
            Inst::Eor => self.a ^= self.read_operand_value().unwrap(),
            Inst::Ora => self.a |= self.read_operand_value().unwrap(),
            Inst::Bit => self.bit(),
            Inst::Anc => {
                self.a &= self.read_operand_value().unwrap();
                self.flags.set(Flags::CARRY, self.a & 0b1000_0000 != 0)
            }
            Inst::Asr => {
                self.a &= self.read_operand_value().unwrap();
                self.a = self.calc_shift(self.a, false, false);
            }
            Inst::Arr => self.arr(),

            Inst::Cmp => self.compare(self.a, None),
            Inst::Cpx => self.compare(self.x, None),
            Inst::Cpy => self.compare(self.y, None),

            // -- Control flow --
            Inst::Jmp => self.pc = self.operand_address.unwrap(),
            Inst::Jsr => self.jsr(),
            Inst::Rts => self.rts(),
            Inst::Brk => self.interrupt(InterruptKind::Brk),
            Inst::Rti => self.rti(),

            Inst::Bcc => self.branch(!self.flags.contains(Flags::CARRY)),
            Inst::Bcs => self.branch(self.flags.contains(Flags::CARRY)),
            Inst::Beq => self.branch(self.flags.contains(Flags::ZERO)),
            Inst::Bmi => self.branch(self.flags.contains(Flags::NEGATIVE)),
            Inst::Bne => self.branch(!self.flags.contains(Flags::ZERO)),
            Inst::Bpl => self.branch(!self.flags.contains(Flags::NEGATIVE)),
            Inst::Bvc => self.branch(!self.flags.contains(Flags::OVERFLOW)),
            Inst::Bvs => self.branch(self.flags.contains(Flags::OVERFLOW)),

            // Does nothing
            Inst::Nop => self.nop(),
            Inst::Hlt => return Err(CpuError::Halted),
        }

        #[rustfmt::skip]
        if matches!(opcode.instruction,
            Inst::Las | Inst::Pla | Inst::Slo | Inst::Rla | Inst::Sre | Inst::Ora | Inst::Eor
                | Inst::And | Inst::Anc
        ) {
            self.set_zero_neg_flags(self.a)
        };
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
        self.bus.write(0x100 + self.sp as u16, value);
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
        self.bus.read(0x100 + self.sp as u16)
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
            self.bus.write(address, value);
        } else {
            // Still a cycle for implied addressing mode
            self.bus.clock();
        };

        let result = self.calc_shift(value, is_left, contains_carry);
        if let Some(address) = self.operand_address {
            self.bus.write(address, result);
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
            self.bus.write(self.operand_address.unwrap(), value); // See shift func
            self.bus.write(self.operand_address.unwrap(), result);
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

    fn interrupt(&mut self, interrupt: InterruptKind) {
        let mut push_flags = self.flags;
        if interrupt == InterruptKind::Brk {
            push_flags |= Flags::BREAK;
            self.pc += 1; // BRK has padding byte
        }

        self.stack_push_u16(self.pc);
        self.stack_push((self.flags | push_flags).bits());
        // If there is a nmi when we're about the load the load vector then the nmi hijacks it
        // No later than cycle 4
        let load_vector = if self.bus.require_nmi() || interrupt == InterruptKind::Nmi {
            0xfffa
        } else {
            0xfffe
        };

        self.bus.clock();
        self.flags.set(Flags::INTERRUPT, true);
        self.pc = self.bus.read_u16(load_vector);
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
            self.bus.read(address);
        } else {
            self.bus.clock();
        }
    }
}
