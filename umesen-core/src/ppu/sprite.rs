use crate::ppu::{Control, Registers};

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Attributes: u8 {
        const PALLETTE = 0b11;
        const BEHIND = 1 << 5;
        const FLIP_HORIZONTAL = 1 << 6;
        const FLIP_VERTICAL = 1 << 7;
    }
}

impl Attributes {
    pub fn palette(&self) -> u8 {
        (*self & Attributes::PALLETTE).bits()
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flag_map = [
            (Attributes::BEHIND, "B"),
            (Attributes::FLIP_HORIZONTAL, "H"),
            (Attributes::FLIP_VERTICAL, "V"),
        ];
        for (flag, name) in flag_map {
            write!(f, "{} ", if self.contains(flag) { name } else { "-" })?;
        }
        write!(f, "{}", self.palette())?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile_number: u8,
    pub attributes: Attributes,
    /// OAM index to check sprite 0 hit
    pub oam_index: u8,
    color_bits_low: u8,
    color_bits_high: u8,
}

impl Sprite {
    pub fn new(oam: &[u8]) -> Self {
        Self {
            x: *oam.get(3).unwrap_or(&0),
            y: *oam.first().unwrap_or(&0),
            tile_number: *oam.get(1).unwrap_or(&0),
            attributes: Attributes::from_bits_truncate(*oam.get(2).unwrap_or(&0)),
            color_bits_low: 0,
            color_bits_high: 0,
            oam_index: 0,
        }
    }

    pub fn y_intersects(&self, scanline: u16, height: u16) -> bool {
        let self_y = self.y as u16;
        scanline >= self_y && scanline < self_y + height
    }

    pub(crate) fn load_shift_bits(&mut self, scanline: u16, registers: &Registers) {
        let mut tile_number = if registers.control.contains(Control::TALL_SPRITES) {
            self.tile_number & 0b1111_1110
        } else {
            self.tile_number
        };

        let table_number = if registers.control.contains(Control::TALL_SPRITES) {
            // Bit zero contains table number when tall sprites
            self.tile_number & 0b1
        } else {
            registers.control.contains(Control::SPRITE_SECOND_TABLE) as u8
        };

        let mut fine_y = scanline - self.y as u16;
        if self.attributes.contains(Attributes::FLIP_VERTICAL) {
            // Flip the fine y
            fine_y = (registers.control.sprite_height() - 1) - fine_y;
        }

        // Go to the next tile if y overflowed tile
        if fine_y >= 8 {
            tile_number += 1;
        }

        let (tile_lsb, tile_msb) =
            registers
                .bus
                .read_pattern_tile_planes(tile_number, table_number, fine_y);

        self.color_bits_low = tile_lsb;
        self.color_bits_high = tile_msb;
    }

    pub(crate) fn color_index(&self, scan_x: usize) -> u8 {
        // Calculate the x position of scan x relative to the sprite x
        let mut x = scan_x.wrapping_sub(self.x as usize);
        if x > 7 {
            return 0;
        }

        if self.attributes.contains(Attributes::FLIP_HORIZONTAL) {
            x = 7 - x;
        }

        crate::ppu::add_bit_planes(self.color_bits_low, self.color_bits_high, 0b1000_0000 >> x)
    }
}
