use std::{cell::RefCell, rc::Rc};

use crate::{
    Cartridge,
    cartridge::{FixedArray, Mirroring},
};

const PALETTE_RAM_SIZE: usize = 0x20;
/// Size of one pattern table in number of tiles, add this to tile number to access the next pattern table
pub const PATTERN_TILE_COUNT: u16 = 256;
pub const PALETTE_START: u16 = 0x3f00;

#[derive(Default)]
pub struct PpuBus {
    pub palette_ram: FixedArray<u8, PALETTE_RAM_SIZE>,
    pub nametable_ram: FixedArray<u8, 0x800>,
    pub(crate) cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl PpuBus {
    pub fn read_u8(&self, mut address: u16) -> u8 {
        address %= 0x4000;
        match address {
            0x0000..=0x1fff => match self.cartridge.as_ref() {
                Some(cartridge) => cartridge.borrow_mut().ppu_read(address),
                None => 0,
            },
            0x2000..=0x3eff => self.nametable_ram[self.mirror_nametable(address)],
            // Palette byte is only 6 bit
            PALETTE_START..=0x3fff => self.palette_ram[mirror_palette(address)],
            _ => unreachable!(),
        }
    }

    pub fn write_u8(&mut self, mut address: u16, value: u8) {
        address %= 0x4000;
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
            PALETTE_START..=0x3fff => self.palette_ram[mirror_palette(address)] = value,
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
        // |||||| |||||+++- T: Fine Y offset, the pixel row number within a tile
        // |||||| ||||+---- P: Bit plane (0: less significant bit; 1: more significant bit)
        // ||++++-++++----- N: Tile number from name table
        // |+-------------- H: Half of pattern table (0: "left"; 1: "right")
        // +--------------- 0: Pattern table is at $0000-$1FFF
        let address = ((tile_number) << 4) | (fine_y % 8);
        (self.read_u8(address), self.read_u8(address + 8))
    }

    fn mirror_nametable(&self, address: u16) -> usize {
        mirror_nametable(
            address,
            self.cartridge
                .as_ref()
                .map(|c| c.borrow().mirroring())
                .unwrap_or_default(),
        )
    }
}

/// Maps the address into nametable ram index
fn mirror_nametable(address: u16, mirroring: Mirroring) -> usize {
    let table_address = address & 0x0fff;

    let table_number = match mirroring {
        Mirroring::Horizontal => match table_address {
            0x000..=0x7ff => 0,
            0x800.. => 1,
        },
        Mirroring::Vertical => match table_address {
            0x000..=0x3ff => 0,
            0x400..=0x7ff => 1,
            0x800..=0xbff => 0,
            0xc00.. => 1,
        },
        Mirroring::SingleScreenLow => 0,
        Mirroring::SingleScreenHigh => 1,
        Mirroring::FourScreen => todo!(),
    };
    (table_number * 0x400 + (address % 0x400)) as usize
}

fn mirror_palette(address: u16) -> usize {
    let address = address as usize % PALETTE_RAM_SIZE;
    match address {
        // Mirror sprite palettes to background
        0x10 | 0x14 | 0x18 | 0x1c => address - 0x10,
        _ => address,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mirroring() {
        use Mirroring::*;
        assert_eq!(mirror_nametable(0x2020, Vertical), 0x0020);
        assert_eq!(mirror_nametable(0x2420, Vertical), 0x0420);
        assert_eq!(mirror_nametable(0x2820, Vertical), 0x0020);
        assert_eq!(mirror_nametable(0x2c20, Vertical), 0x0420);

        assert_eq!(mirror_nametable(0x3020, Horizontal), 0x0020);
        assert_eq!(mirror_nametable(0x3420, Horizontal), 0x0020);
        assert_eq!(mirror_nametable(0x3820, Horizontal), 0x0420);
        assert_eq!(mirror_nametable(0x3c20, Horizontal), 0x0420);

        assert_eq!(mirror_nametable(0x2020, SingleScreenLow), 0x0020);
        assert_eq!(mirror_nametable(0x2420, SingleScreenLow), 0x0020);
        assert_eq!(mirror_nametable(0x2820, SingleScreenLow), 0x0020);
        assert_eq!(mirror_nametable(0x2c20, SingleScreenLow), 0x0020);

        assert_eq!(mirror_nametable(0x2020, SingleScreenHigh), 0x0420);
        assert_eq!(mirror_nametable(0x2420, SingleScreenHigh), 0x0420);
        assert_eq!(mirror_nametable(0x2820, SingleScreenHigh), 0x0420);
        assert_eq!(mirror_nametable(0x2c20, SingleScreenHigh), 0x0420);
    }
}
