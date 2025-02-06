use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::{Cartridge, FixedArray, MemoryBankExt},
    Controller, Ppu,
};

#[derive(Default)]
pub struct CpuBus {
    // 2kb of cpu ram
    pub ram: FixedArray<u8, 0x800>,
    /// Number of cycles for the cpu to wait before executing the next instruction
    /// (aka number of cycles added when executing the previous instruction)
    pub cpu_cycles_to_wait: u32,
    /// Cpu cycles counter for debugging
    pub cpu_cycles_total: u32,
    pub ppu: Ppu,
    pub cartridge: Option<Rc<RefCell<Cartridge>>>,
    pub open_bus: u8,
    pub controllers: [Controller; 2],
}

impl CpuBus {
    /// Immutable read function for peeking into memory
    /// Reads into some address cause side effects
    pub fn immut_read_u8(&self, address: u16) -> u8 {
        if let Some(cart) = self.cartridge.as_ref() {
            if let Some(byte) = cart.borrow().cpu_read(address) {
                return byte;
            }
        }

        // https://www.nesdev.org/wiki/CPU_memory_map
        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram.mirrored_read(address & 0x7ff),
            0x2000..=0x3fff => self.ppu.registers.immut_read_u8(address),
            _ => self.open_bus,
        }
    }

    pub fn read_u8(&mut self, address: u16) -> u8 {
        let output = match address {
            0x2000..=0x3fff => self.ppu.registers.read_u8(address),
            0x4016 => self.controllers[0].read_u8(),
            0x4017 => self.controllers[1].read_u8(),
            _ => self.immut_read_u8(address),
        };
        self.open_bus = output;
        self.clock();
        output
    }

    pub fn write_u8(&mut self, address: u16, value: u8) {
        if let Some(cartridge) = self.cartridge.as_ref() {
            cartridge.borrow_mut().cpu_write(address, value);
        }

        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram.mirrored_write(address, value),
            0x2000..=0x3fff => self.ppu.registers.write_u8(address, value),
            0x4016 => {
                self.controllers[0].write_u8(value);
                self.controllers[1].write_u8(value);
            }
            _ => (),
        }
        self.clock();
    }

    pub fn read_u16(&mut self, address: u16) -> u16 {
        let lsb = self.read_u8(address) as u16;
        let msb = self.read_u8(address + 1) as u16;
        (msb << 8) | lsb
    }

    /// Same as read u16 but the high byte is wrapped to the beggining of the page
    pub fn read_u16_wrapped(&mut self, address: u16) -> u16 {
        let lsb = self.read_u8(address) as u16;
        // Wrap the page by always getting the address high byte from the current page
        let address_for_msb = (address & 0xff00) | ((address + 1) & 0x00ff);
        let msb = self.read_u8(address_for_msb) as u16;
        (msb << 8) | lsb
    }

    pub fn write_u16(&mut self, address: u16, value: u16) {
        let lsb = value as u8;
        let msb = (value << 8) as u8;
        self.write_u8(address, lsb);
        self.write_u8(address + 1, msb);
    }

    // Clock all devices on the cpu bus relative to a cpu cycle
    pub fn clock(&mut self) {
        self.cpu_cycles_to_wait += 1;
        self.cpu_cycles_total += 1;
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
