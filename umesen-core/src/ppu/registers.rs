bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    struct Control: u8 {
        /// Bit 8 of the X scroll position
        const X_SCROLL_HIGH_BIT = 1;
        /// Bit 8 of the Y scroll position
        const Y_SCROLL_HIGH_BIT = 1 << 1;
        /// (0: add 1 going across, 1: add 32 going down)
        const VRAM_INCREMENT = 1 << 2;
        /// (0: 0x0000, 1: 0x1000)
        const SPRITE_TABLE_OFFSET = 1 << 3;
        /// (0: 0x0000, 1: 0x1000)
        const BACKGROUND_TABLE_OFFSET = 1 << 4;
        /// (0: 8x8 pixels, 1: 8x16 pixels)
        const TALL_SPRITES = 1 << 5;
        /// (0: read backdrop from EXT pins, 1: output color on EXT pins)
        const PPU_SELECT = 1 << 6;
        const VBLANK_NMI = 1 << 7;
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    struct Mask: u8 {
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

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    struct Status: u8 {
        const SPRITE_OVERFLOW = 1 << 6;
        const SPRITE_0_HIT = 1 << 7;
        const VBLANK = 1 << 7;
    }
}

#[derive(Default)]
pub struct Registers {
    control: Control,
    mask: Mask,
    status: Status,
    oam_address: u8,
    oam_data: u8,
    /// Low 8 bits of the x scroll
    scroll_x: u8,
    /// Low 9 bits of the y scroll
    scroll_y: u8,
}

impl Registers {
    pub fn read(&self, address: u16) -> u8 {
        debug_assert!((0x2000..=0x3fff).contains(&address));
        match address % 8 {
            0 => 0,
            1 => 0,
            2 => self.status.bits(),
            3 => 0,
            4 => self.oam_data,
            5 => 0,
            6 => 0,
            7 => 0,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        debug_assert!((0x2000..=0x3fff).contains(&address));
        match address % 8 {
            0 => self.control = Control::from_bits_retain(value),
            1 => self.mask = Mask::from_bits_retain(value),
            2 => (),
            3 => self.oam_address = value,
            4 => {
                self.oam_data = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            5 => (),
            6 => (),
            7 => (),
            _ => unreachable!(),
        }
    }
}
