use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::{FixedArray, Mirroring},
    Cartridge,
};

const PALETTE_RAM_SIZE: usize = 0x20;
const NAMETABLE_RAM_SIZE: usize = 0x800;
/// Size of one pattern table in number of tiles (aka one byte), add this to tile number to access the next pattern table
pub const PATTERN_TILE_COUNT: u16 = 256;

#[derive(Clone, Default)]
pub struct PpuBus {
    pub palette_ram: FixedArray<u8, PALETTE_RAM_SIZE>,
    pub nametable_ram: FixedArray<u8, NAMETABLE_RAM_SIZE>,
    pub(crate) cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl PpuBus {
    pub fn read_u8(&self, address: u16) -> u8 {
        match address % 0x4000 {
            0x0000..=0x1fff => match self.cartridge.as_ref() {
                Some(cartridge) => cartridge.borrow().ppu_read(address),
                None => 0,
            },
            0x2000..=0x3eff => self.nametable_ram[self.mirror_nametable(address)],
            0x3f00..=0x3fff => self.palette_ram[mirror_palette(address)],
            _ => unreachable!(),
        }
    }

    pub fn write_u8(&mut self, address: u16, value: u8) {
        match address % 0x4000 {
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
            _ => unreachable!(),
        }
    }

    /// Gets the pattern table tile planes
    /// Return (lsb plane, msb plane)
    pub fn read_pattern_tile_planes(&self, tile_number: u16, fine_y: u16) -> (u8, u8) {
        // From nes wiki: https://www.nesdev.org/wiki/PPU_pattern_tables#Addressing
        // DCBA98 76543210
        // ---------------
        // 0HNNNN NNNNPyyy
        // |||||| |||||+++- T: Fine Y offset, the row number within a tile
        // |||||| ||||+---- P: Bit plane (0: less significant bit; 1: more significant bit)
        // ||++++-++++----- N: Tile number from name table
        // |+-------------- H: Half of pattern table (0: "left"; 1: "right")
        // +--------------- 0: Pattern table is at $0000-$1FFF
        let address = ((tile_number) << 4) | (fine_y % 8);
        (self.read_u8(address), self.read_u8(address + 8))
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

fn mirror_nametable(address: u16, mirroring: Mirroring) -> usize {
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
