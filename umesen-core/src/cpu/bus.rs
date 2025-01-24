use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::{Cartridge, FixedArray, MemoryBankExt},
    Ppu,
};

#[derive(Default)]
pub struct CpuBus {
    // 2kb of cpu ram
    pub ram: FixedArray<u8, 2048>,
    /// Cpu cycles counter for debugging
    pub cpu_cycles: u32,
    pub ppu: Ppu,
    pub cartridge: Option<Rc<RefCell<Cartridge>>>,
    pub open_bus: u8,
}

impl std::fmt::Display for CpuBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..0x1000 {
            let line_address_start = i * 0x10;
            write!(f, "${line_address_start:04x}:")?;

            for i in 0..0x10 {
                let byte = self.immut_read_byte(line_address_start + i);
                write!(f, " {byte:02x}")?;
            }

            if i != 0xfff {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl CpuBus {
    /// Immutable read function for peeking into memory
    pub fn immut_read_byte(&self, address: u16) -> u8 {
        // https://www.nesdev.org/wiki/CPU_memory_map
        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram.mirrored_read(address),
            0x2000..=0x3fff => self.ppu.registers.immut_read(address),
            0x4020..=0xffff => match self.cartridge.as_ref() {
                Some(cartridge) => cartridge.borrow().cpu_read(address),
                None => self.open_bus,
            },
            _ => self.open_bus,
        }
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        let output = match address {
            0x2000..=0x3fff => self.ppu.registers.read_byte(address),
            _ => self.immut_read_byte(address),
        };
        self.open_bus = output;
        self.clock();
        output
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram.mirrored_write(address, value),
            0x2000..=0x3fff => self.ppu.registers.write_byte(address, value),
            0x4020..=0xffff => {
                if let Some(cartridge) = self.cartridge.as_ref() {
                    cartridge.borrow_mut().cpu_write(address, value);
                }
            }
            _ => (),
        }
        self.clock();
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let lsb = self.read_byte(address) as u16;
        let msb = self.read_byte(address + 1) as u16;
        (msb << 8) | lsb
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let lsb = value as u8;
        let msb = (value << 8) as u8;
        self.write_byte(address, lsb);
        self.write_byte(address + 1, msb);
    }

    // Clock all devices on the cpu bus relative to a cpu cycle
    pub fn clock(&mut self) {
        self.cpu_cycles += 1;
        for _ in 0..3 {
            self.ppu.clock();
        }
    }

    pub fn attach_catridge(&mut self, catridge: Cartridge) {
        let catridge_rc = Rc::new(RefCell::new(catridge));
        self.cartridge = Some(catridge_rc.clone());
        self.ppu.registers.bus.cartridge = Some(catridge_rc);
    }

    pub fn require_nmi(&mut self) -> bool {
        let status = self.ppu.require_nmi;
        self.ppu.require_nmi = false;
        status
    }
}
