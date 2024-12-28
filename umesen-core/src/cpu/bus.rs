use std::{cell::RefCell, rc::Rc};

use crate::cartridge::CartridgeBoard;

#[derive(Clone)]
pub struct CpuBus {
    // 2kb of cpu ram
    pub ram: [u8; 2048],
    /// Cpu cycles counter for debugging
    pub cpu_cycles: u32,
    pub cartridge: Option<Rc<RefCell<dyn CartridgeBoard>>>,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 2048],
            cpu_cycles: 0,
            cartridge: None,
        }
    }
}

impl CpuBus {
    pub fn read_byte(&mut self, address: u16) -> u8 {
        self.clock();
        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram[(address as usize) % self.ram.len()],
            0x4020..=0xffff => {
                if let Some(cartridge) = self.cartridge.as_ref() {
                    cartridge.borrow().prg_read(address)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.clock();
        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram[(address as usize) % self.ram.len()] = value,
            0x4020..=0xffff => {
                if let Some(cartridge) = self.cartridge.as_mut() {
                    cartridge.borrow_mut().prg_write(address, value);
                }
            }
            _ => (),
        }
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let lsb = self.read_byte(address) as u16;
        let msb = self.read_byte(address + 1) as u16;
        msb << 8 | lsb
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let lsb = value as u8;
        let msb = (value << 8) as u8;
        self.write_byte(address, lsb);
        self.write_byte(address + 1, msb);
    }

    pub fn clock(&mut self) {
        // Every cpu clock is 12 master clocks
        self.cpu_cycles += 1;
    }
}
