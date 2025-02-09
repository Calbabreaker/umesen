use crate::{
    cartridge::FixedArray,
    ppu::{bus::PpuBus, PATTERN_TILE_COUNT},
};

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Control: u8 {
        /// XY bits of nametable or each unit is 0x400 offset
        const NAMETABLE = 0b11;
        /// 0: add 1, 1: add 32
        const VRAM_INCREMENT = 1 << 2;
        /// Use second pattern table if set
        const SPRITE_SECOND_TABLE = 1 << 3;
        /// Use second pattern table if set
        const BACKGROUND_SECOND_TABLE = 1 << 4;
        /// 0: 8x8 pixels, 1: 8x16 pixels
        const TALL_SPRITES = 1 << 5;
        /// 0: read backdrop from EXT pins, 1: output color on EXT pins
        const PPU_SELECT = 1 << 6;
        /// Enable sending NMI on vblank
        const VBLANK_NMI = 1 << 7;
    }
}

impl Control {
    pub fn sprite_height(&self) -> u16 {
        if self.contains(Control::TALL_SPRITES) {
            16
        } else {
            8
        }
    }

    pub fn background_table_offset(&self) -> u16 {
        self.contains(Control::BACKGROUND_SECOND_TABLE) as u16 * PATTERN_TILE_COUNT
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Mask: u8 {
        const GRAYSCALE = 1;
        /// Show background in leftmost 8 pixels of screen
        const SHOW_BACKGROUND_LEFTMOST_8 = 1 << 1;
        /// Show sprite in leftmost 8 pixels of screen
        const SHOW_SPRITE_LEFTMOST_8 = 1 << 2;
        /// Enable rendering of background
        const RENDER_BACKGROUND = 1 << 3;
        /// Enable rendering of sprite
        const RENDER_SPRITE = 1 << 4;
        const EMPHASIZE_RED = 1 << 5;
        const EMPHASIZE_GREEN = 1 << 6;
        const EMPHASIZE_BLUE = 1 << 7;
    }
}

impl Mask {
    /// Is rendering sprite or background
    pub fn is_rendering(&self) -> bool {
        self.intersects(Mask::RENDER_SPRITE | Mask::RENDER_BACKGROUND)
    }

    pub fn can_show_sprite(&self, scan_x: usize) -> bool {
        let can_show_leftmost = self.contains(Mask::SHOW_SPRITE_LEFTMOST_8) || scan_x >= 8;
        self.contains(Mask::RENDER_SPRITE) && can_show_leftmost
    }

    pub fn can_show_background(&self, scan_x: usize) -> bool {
        let can_show_leftmost = self.contains(Mask::SHOW_BACKGROUND_LEFTMOST_8) || scan_x >= 8;
        self.contains(Mask::RENDER_BACKGROUND) && can_show_leftmost
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Status: u8 {
        const SPRITE_OVERFLOW = 1 << 5;
        const SPRITE_0_HIT = 1 << 6;
        const VBLANK = 1 << 7;
    }
}

/// Internal 15-bit registers (t and v) used for rendering and memory access
/// These can act as a 15-bit address to access the ppu bus or a packed bitfield
/// From nesdev wiki: https://www.nesdev.org/wiki/PPU_scrolling
/// 0yyyNNYY YYYXXXXX
///  ||||||| |||+++++---- coarse X scroll
///  |||||++-+++--------- coarse Y scroll
///  |||++--------------- nametable select X and y
///  +++----------------- fine Y scroll
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TvRegister(pub u16);

#[rustfmt::skip]
impl TvRegister {
    // Select bits
    pub const COARSE_X:    u16 = 0b00000000_00011111;
    pub const COARSE_Y:    u16 = 0b00000011_11100000;
    pub const NAMETABLE:   u16 = 0b00001100_00000000;
    pub const NAMETABLE_X: u16 = 0b00000100_00000000;
    pub const NAMETABLE_Y: u16 = 0b00001000_00000000;
    pub const FINE_Y:      u16 = 0b01110000_00000000;
    pub const LOW:         u16 = 0b00000000_11111111;
    pub const HIGH:        u16 = 0b11111111_00000000;
}

impl TvRegister {
    pub fn set(&mut self, select_bits: u16, value: impl Into<u16>) {
        let value = value.into();
        // Check value fits into the bits
        debug_assert!(value <= (select_bits >> select_bits.trailing_zeros()));

        let value_shifted = value << select_bits.trailing_zeros();
        self.0 = value_shifted | (self.0 & (!select_bits));
    }

    pub fn get(&self, select_bits: u16) -> u16 {
        let value = self.0 & select_bits;
        value >> select_bits.trailing_zeros()
    }

    pub fn nametable_address(&self) -> u16 {
        // Bits in register are ordered in such a way the bottom 12 can index into the nametable automatically
        0x2000 + (self.0 & 0x0fff)
    }

    pub fn attribute_address(&self) -> u16 {
        let tile_x = self.get(Self::COARSE_X) / 4;
        let tile_y = self.get(Self::COARSE_Y) / 4;
        let attribute_number = tile_y * 8 + tile_x;
        let nametable_offset = 0x2000 + (self.0 & Self::NAMETABLE);
        nametable_offset + 0x3c0 + attribute_number
    }

    /// Shift an attribute byte to get the palette id into the first two bits based on the coarse xy
    pub fn shift_attribute(&self, attribute: u8) -> u8 {
        let quadrant_x = (self.get(Self::COARSE_X) % 4) / 2;
        let quadrant_y = (self.get(Self::COARSE_Y) % 4) / 2;
        let shift = (quadrant_x + quadrant_y * 2) * 2;
        (attribute >> shift) & 0b11
    }

    pub fn scroll_coarse_x(&mut self) {
        if self.scroll_wrap(Self::COARSE_X, 31) {
            // Flip nametable x bit to wrap around
            self.0 ^= Self::NAMETABLE_X;
        }
    }

    pub fn scroll_fine_y(&mut self) {
        if self.scroll_wrap(Self::FINE_Y, 7) {
            // Scroll coarse y wrappiong at 30 since bottom is taken by attribute data
            if self.scroll_wrap(Self::COARSE_Y, 29) {
                self.0 ^= Self::NAMETABLE_Y;
            } else if self.get(Self::COARSE_Y) == 31 {
                // 30-31 is invalid but still needs to be wrapped at 32
                self.set(Self::COARSE_Y, 0u8);
            }
        }
    }

    pub fn set_x(&mut self, other: &TvRegister) {
        let bits = TvRegister::COARSE_X | TvRegister::NAMETABLE_X;
        self.set(bits, other.get(bits));
    }

    pub fn set_y(&mut self, other: &TvRegister) {
        let bits = TvRegister::COARSE_Y | TvRegister::NAMETABLE_Y | TvRegister::FINE_Y;
        self.set(bits, other.get(bits));
    }

    // Increment a value, wrapping at wrap and returning true
    fn scroll_wrap(&mut self, select_bits: u16, wrap: u16) -> bool {
        let value = self.get(select_bits);
        if value == wrap {
            self.set(select_bits, 0u8);
            true
        } else {
            self.set(select_bits, value + 1);
            false
        }
    }
}

impl std::fmt::Display for TvRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CX: {}, CY: {}, NXY: {:02b}, FY: {}",
            self.get(TvRegister::COARSE_X),
            self.get(TvRegister::COARSE_Y),
            self.get(TvRegister::NAMETABLE),
            self.get(TvRegister::FINE_Y)
        )
    }
}

#[derive(Default)]
pub struct Registers {
    pub bus: PpuBus,
    pub control: Control,
    pub mask: Mask,
    pub status: Status,
    pub t: TvRegister,
    pub v: TvRegister,
    pub latch: bool,
    pub fine_x: u8,
    pub oam_address: u8,
    pub oam_data: FixedArray<u8, 256>,
    read_buffer: u8,
    open_bus: u8,
}

impl Registers {
    pub(crate) fn immut_read_u8(&self, address: u16) -> u8 {
        debug_assert!((0x2000..=0x3fff).contains(&address));
        match address % 8 {
            // Get status bits and fill unused with open bus
            2 => self.status.bits() | (self.open_bus & (!Status::all().bits())),
            4 => self.oam_data[self.oam_address as usize],
            // Get PPU memory data
            7 => {
                // Palette address gets data returned immediately instead of being buffered
                if self.v.0 >= 0x3f00 {
                    self.bus.read_u8(self.v.0)
                } else {
                    self.read_buffer
                }
            }
            _ => self.open_bus,
        }
    }

    pub(crate) fn read_u8(&mut self, address: u16) -> u8 {
        let output = self.immut_read_u8(address);
        match address % 8 {
            2 => {
                // Reset latch when read for real
                self.status.remove(Status::VBLANK);
                self.latch = false;
            }
            7 => {
                self.read_buffer = self.bus.read_u8(self.v.0);
                self.increment_v_register();
            }
            _ => (),
        }
        self.open_bus = output;
        output
    }

    pub(crate) fn write_u8(&mut self, address: u16, value: u8) {
        debug_assert!((0x2000..=0x3fff).contains(&address));
        self.open_bus = value;
        match address % 8 {
            0 => self.write_control(value),
            1 => self.mask = Mask::from_bits(value).unwrap(),
            2 => (),
            3 => self.oam_address = value,
            4 => self.write_oam_data(value),
            5 => self.write_scroll(value),
            6 => self.write_vram_address(value),
            7 => self.write_vram_data(value),
            _ => unreachable!(),
        }
    }

    pub fn write_oam_data(&mut self, mut value: u8) {
        // Zero out unused bits when set the attribute byte of oam
        if self.oam_address % 4 == 2 {
            value &= 0b1110_0011;
        }

        self.oam_data[self.oam_address as usize] = value;
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    fn write_control(&mut self, value: u8) {
        self.control = Control::from_bits(value).unwrap();
        let nametable_bits = (self.control & Control::NAMETABLE).bits();
        self.t.set(TvRegister::NAMETABLE, nametable_bits);
    }

    fn write_scroll(&mut self, value: u8) {
        let fine = value & 0b111;
        let coarse = value >> 3;
        if !self.latch {
            // X scroll
            self.t.set(TvRegister::COARSE_X, coarse);
            self.fine_x = fine;
        } else {
            // Y Scroll
            self.t.set(TvRegister::COARSE_Y, coarse);
            self.t.set(TvRegister::FINE_Y, fine);
        }
        self.latch = !self.latch;
    }

    fn write_vram_address(&mut self, value: u8) {
        if !self.latch {
            self.t.set(TvRegister::HIGH, value);
        } else {
            // Write low byte and copy to v_register
            self.t.set(TvRegister::LOW, value);
            self.v = self.t;
        }
        self.latch = !self.latch;
    }

    fn write_vram_data(&mut self, value: u8) {
        self.bus.write_u8(self.v.0, value);
        self.increment_v_register();
    }

    fn increment_v_register(&mut self) {
        let amount = if self.control.contains(Control::VRAM_INCREMENT) {
            32
        } else {
            1
        };
        self.v.0 = self.v.0.wrapping_add(amount)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tv_register() {
        let mut register = TvRegister::default();
        register.set(TvRegister::COARSE_X, 10u8);
        register.set(TvRegister::COARSE_Y, 15u8);
        register.set(TvRegister::NAMETABLE, 1u8);
        assert_eq!(register.get(TvRegister::COARSE_X), 10);
        assert_eq!(register.get(TvRegister::COARSE_Y), 15);
        assert_eq!(register.nametable_address(), 0x2400 + 490);
        assert_eq!(register.attribute_address(), 0x27da);
    }
}
