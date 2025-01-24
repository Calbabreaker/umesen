use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::FixedArray, Cartridge};

const PALETTE_RAM_SIZE: usize = 32;
const NAMETABLE_RAM_SIZE: usize = 2048;

#[derive(Default)]
pub struct PpuBus {
    palette_ram: FixedArray<u8, NAMETABLE_RAM_SIZE>,
    nametable_ram: FixedArray<u8, NAMETABLE_RAM_SIZE>,
    pub cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl PpuBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => match self.cartridge.as_ref() {
                Some(cartridge) => cartridge.borrow().ppu_read(address),
                None => 0,
            },
            0x2000..=0x3eff => self.nametable_ram[mirror_nametable_address(address)],
            0x3f00..=0x3fff => self.palette_ram[mirror_palette_address(address)],
            _ => 0,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => {
                if let Some(cartridge) = self.cartridge.as_ref() {
                    cartridge.borrow_mut().ppu_write(address, value);
                }
            }
            0x2000..=0x3eff => self.nametable_ram[mirror_nametable_address(address)] = value,
            0x3f00..=0x3fff => self.palette_ram[mirror_palette_address(address)] = value,
            _ => (),
        }
    }
}

fn mirror_nametable_address(address: u16) -> usize {
    0
}

fn mirror_palette_address(address: u16) -> usize {
    let address = address as usize % PALETTE_RAM_SIZE;
    match address {
        0x10 | 0x14 | 0x18 | 0x1c => address - 0x10,
        _ => address,
    }
}
