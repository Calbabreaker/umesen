use crate::{
    cartridge::FixedArray,
    ppu::{
        HEIGHT, PALETTE_START, PATTERN_TILE_COUNT, PRERENDER_SCANLINE, Sprite, VramRegister, WIDTH,
        bus::PpuBus, sprite::Attributes,
    },
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
    pub fn sprite_height(&self) -> u8 {
        if self.contains(Control::TALL_SPRITES) {
            16
        } else {
            8
        }
    }

    /// Get the offset of the currently selected background pattern table in number of tiles
    /// Will be 256 if BACKGROUND_SECOND_TABLE is set
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

/// AKA how many frames before open bus decays to 0
const OPEN_BUS_DECAY_START: u32 = 30;

#[derive(Clone, Default)]
pub struct Registers {
    pub bus: PpuBus,
    pub control: Control,
    pub mask: Mask,
    pub status: Status,
    pub t: VramRegister,
    pub v: VramRegister,
    pub latch: bool,
    pub fine_x: u8,
    pub oam_address: u8,
    pub oam_data: FixedArray<u8, 256>,
    pub read_buffer: u8,
    pub open_bus: u8,
    open_bus_decay_counter: u32,

    pub scanline: usize,
    pub dot: usize,
    pub frame_count: u32,
}

impl Registers {
    pub(crate) fn immut_read_u8(&self, address: u16) -> u8 {
        std::debug_assert_matches!(address, 0x2000..=0x3fff);
        match address % 8 {
            // Get status bits and fill unused with open bus
            2 => self.status.bits() | (self.open_bus & (!Status::all().bits())),
            4 => self.read_oam_data(),
            7 => self.read_buffer,
            _ => self.open_bus,
        }
    }

    pub(crate) fn read_u8(&mut self, address: u16) -> u8 {
        let mut output = self.immut_read_u8(address);
        match address % 8 {
            2 => {
                // Reset latch when read for real
                self.status.remove(Status::VBLANK);
                self.latch = false;
            }
            7 => {
                // Palette address gets data returned immediately instead of being buffered
                // but read_buffer populated with nametable data
                if self.v.0 >= PALETTE_START {
                    output = self.read_palette_ram(self.v.0);
                    self.read_buffer = self.bus.read_u8(0x2f00 | (self.v.0 & 0xff));
                } else {
                    self.read_buffer = self.bus.read_u8(self.v.0);
                }
                self.increment_v_register();
            }
            _ => (),
        }
        self.open_bus = output;
        self.open_bus_decay_counter = OPEN_BUS_DECAY_START;
        output
    }

    pub(crate) fn write_u8(&mut self, address: u16, value: u8) {
        std::debug_assert_matches!(address, 0x2000..=0x3fff);
        self.open_bus = value;
        self.open_bus_decay_counter = OPEN_BUS_DECAY_START;
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

    /// Get the sprite at the oam index with an individual byte offset
    pub fn get_oam_sprite(&self, index: usize, offset: usize) -> Option<Sprite> {
        let i = index * 4 + offset;
        if i < self.oam_data.len() {
            let right_bound = (i + 4).min(self.oam_data.len());
            Some(Sprite::new(&self.oam_data[i..right_bound], index as u8))
        } else {
            None
        }
    }

    pub fn on_visble_dot(&self) -> bool {
        self.dot >= 1 && (self.dot - 1 < WIDTH)
    }

    pub fn read_palette_ram(&self, offset: u16) -> u8 {
        let address = PALETTE_START | (offset & 0xff);
        // Get the palette ram and with open bus
        let mut value = (self.bus.read_u8(address) & 0b0011_1111) | (self.open_bus & 0b1100_0000);
        if self.mask.contains(Mask::GRAYSCALE) {
            value &= 0x30;
        }
        value
    }

    pub(crate) fn next_dot(&mut self) {
        self.dot += 1;
        if self.dot == 341 {
            self.dot = 0;
            self.scanline += 1;
        }

        if self.scanline == PRERENDER_SCANLINE + 1 {
            self.frame_count = self.frame_count.wrapping_add(1);
            self.scanline = 0;
            if self.open_bus_decay_counter == 0 {
                self.open_bus = 0;
            } else {
                self.open_bus_decay_counter -= 1;
            }
        }
    }

    // TODO: weird oam behaviour while reading/writing during rendering
    fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_address as usize]
    }

    pub(crate) fn write_oam_data(&mut self, mut value: u8) {
        // Zero out unused bits when setting the attribute byte of oam
        if self.oam_address % 4 == 2 {
            value &= Attributes::all().bits();
        }

        self.oam_data[self.oam_address as usize] = value;
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    fn write_control(&mut self, value: u8) {
        self.control = Control::from_bits(value).unwrap();
        let nametable_bits = (self.control & Control::NAMETABLE).bits();
        self.t.set(VramRegister::NAMETABLE, nametable_bits);
    }

    fn write_scroll(&mut self, value: u8) {
        let fine = value & 0b111;
        let coarse = value >> 3;
        if !self.latch {
            // X scroll
            self.t.set(VramRegister::COARSE_X, coarse);
            self.fine_x = fine;
        } else {
            // Y Scroll
            self.t.set(VramRegister::COARSE_Y, coarse);
            self.t.set(VramRegister::FINE_Y, fine);
        }
        self.latch = !self.latch;
    }

    fn write_vram_address(&mut self, value: u8) {
        if !self.latch {
            self.t.set(VramRegister::HIGH, value);
        } else {
            self.t.set(VramRegister::LOW, value);
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
        self.v.0 = self.v.0.wrapping_add(amount);
        // Weird increment behaviour when ppu is rendering
        if self.on_visble_dot() && self.scanline < HEIGHT && self.mask.is_rendering() {
            self.v.scroll_fine_y();
            self.v.scroll_coarse_x();
        }
    }
}
