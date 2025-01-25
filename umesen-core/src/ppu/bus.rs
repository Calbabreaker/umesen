use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::{FixedArray, Mirroring},
    Cartridge,
};

const PALETTE_RAM_SIZE: usize = 0x20;
const NAMETABLE_RAM_SIZE: usize = 0x800;

#[derive(Default)]
pub struct PpuBus {
    pub palette_ram: FixedArray<u8, PALETTE_RAM_SIZE>,
    pub nametable_ram: FixedArray<u8, NAMETABLE_RAM_SIZE>,
    pub(crate) cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl PpuBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        debug_assert!((0x0000..=0x3fff).contains(&address));
        match address {
            0x0000..=0x1fff => match self.cartridge.as_ref() {
                Some(cartridge) => cartridge.borrow().ppu_read(address),
                None => 0,
            },
            0x2000..=0x3eff => self.nametable_ram[self.mirror_nametable(address)],
            0x3f00..=0x3fff => self.palette_ram[mirror_palette(address)],
            _ => 0,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        debug_assert!((0x0000..=0x3fff).contains(&address));
        match address {
            0x0000..=0x1fff => {
                if let Some(cartridge) = self.cartridge.as_ref() {
                    cartridge.borrow_mut().ppu_write(address, value);
                }
            }
            0x2000..=0x3eff => {
                let address = self.mirror_nametable(address);
                self.nametable_ram[address] = value;
            }
            0x3f00..=0x3fff => self.palette_ram[mirror_palette(address)] = value,
            _ => (),
        }
    }

    fn mirror_nametable(&self, address: u16) -> usize {
        let mirroring = if let Some(cartridge) = self.cartridge.as_ref() {
            cartridge.borrow().mirroring()
        } else {
            Mirroring::default()
        };
        mirror_nametable(address, mirroring)
    }
}

pub fn mirror_nametable(address: u16, mirroring: Mirroring) -> usize {
    let address = address as usize % 0x1000;
    match mirroring {
        Mirroring::Horizontal => {
            let nametable_offset = (address / NAMETABLE_RAM_SIZE) * (NAMETABLE_RAM_SIZE / 2);
            let tile_offset = address % (NAMETABLE_RAM_SIZE / 2);
            nametable_offset + tile_offset
        }
        Mirroring::Vertical => address % NAMETABLE_RAM_SIZE,
    }
}

fn mirror_palette(address: u16) -> usize {
    let address = address as usize % PALETTE_RAM_SIZE;
    match address {
        0x10 | 0x14 | 0x18 | 0x1c => address - 0x10,
        _ => address,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mirroring() {
        assert_eq!(mirror_nametable(0x2020, Mirroring::Vertical), 0x0020);
        assert_eq!(mirror_nametable(0x2420, Mirroring::Vertical), 0x0420);
        assert_eq!(mirror_nametable(0x2820, Mirroring::Vertical), 0x0020);
        assert_eq!(mirror_nametable(0x2c20, Mirroring::Vertical), 0x0420);

        assert_eq!(mirror_nametable(0x2020, Mirroring::Horizontal), 0x0020);
        assert_eq!(mirror_nametable(0x2420, Mirroring::Horizontal), 0x0020);
        assert_eq!(mirror_nametable(0x2820, Mirroring::Horizontal), 0x0420);
        assert_eq!(mirror_nametable(0x2c20, Mirroring::Horizontal), 0x0420);
    }
}
