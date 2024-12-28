use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::CartridgeBoard, Cartridge, CartridgeData};

#[derive(Clone)]
pub struct CpuBus {
    // 2kb of cpu ram
    pub ram: [u8; 2048],
    /// Cpu cycles counter for debugging
    pub cpu_cycles: u32,
    pub cartridge: Rc<RefCell<dyn CartridgeBoard>>,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self::new(Cartridge::new(CartridgeData::default()).unwrap())
    }
}

impl CpuBus {
    pub fn new(cartridge: Rc<RefCell<dyn CartridgeBoard>>) -> Self {
        Self {
            ram: [0; 2048],
            cpu_cycles: 0,
            cartridge,
        }
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        self.clock();
        *self.index_byte(address as usize)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.clock();
        *self.index_byte(address as usize) = value;
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

    // Cpu memory map: https://www.nesdev.org/wiki/CPU_memory_map
    fn index_byte(&mut self, address: usize) -> &mut u8 {
        match address as u16 {
            // 2kb of ram is mirrored 3 times
            // 0x0000..=0x1fff => self.ram.get_mut(address & 0x07ff).unwrap(),
            0x0000..=0xffff => self.ram.get_mut(address & 0x07ff).unwrap(),
            // 0x2000..=0x3fff => self.ppu.write_register(address & 0x0007, data),
            // _ => ,
        }
    }
}
