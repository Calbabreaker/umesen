use crate::ppu::bus::PpuBus;

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Control: u8 {
        /// Bit 8 of the X scroll position
        const X_SCROLL_HIGH_BIT = 1;
        /// Bit 8 of the Y scroll position
        const Y_SCROLL_HIGH_BIT = 1 << 1;
        /// 0: add 1 going across, 1: add 32 going down
        const VRAM_INCREMENT = 1 << 2;
        /// 0: 0x0000, 1: 0x1000
        const SPRITE_TABLE_OFFSET = 1 << 3;
        /// 0: 0x0000, 1: 0x1000
        const BACKGROUND_TABLE_OFFSET = 1 << 4;
        /// 0: 8x8 pixels, 1: 8x16 pixels
        const TALL_SPRITES = 1 << 5;
        /// 0: read backdrop from EXT pins, 1: output color on EXT pins
        const PPU_SELECT = 1 << 6;
        /// Enable sendiing NMI on vblank
        const VBLANK_NMI = 1 << 7;
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

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Status: u8 {
        const SPRITE_OVERFLOW = 1 << 6;
        const SPRITE_0_HIT = 1 << 7;
        const VBLANK = 1 << 7;
    }
}

#[derive(Default)]
pub struct Registers {
    pub bus: PpuBus,
    pub control: Control,
    pub mask: Mask,
    pub status: Status,
    oam_address: u8,
    oam_data: u8,
    latch: u8,
    read_buffer: u8,
    vram_address: u16,
    open_bus: u8,
}

impl Registers {
    pub(crate) fn immut_read(&self, address: u16) -> u8 {
        debug_assert!((0x2000..=0x3fff).contains(&address));
        match address % 8 {
            0 => self.open_bus,
            1 => self.open_bus,
            // Fill the unused bits with open bus
            2 => self.status.bits() | (self.open_bus & (!Status::all().bits())),
            3 => self.open_bus,
            4 => self.oam_data,
            5 => self.open_bus,
            6 => self.open_bus,
            7 => {
                if address >= 0x3f00 {
                    self.bus.read_byte(address)
                } else {
                    self.read_buffer
                }
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn read_byte(&mut self, address: u16) -> u8 {
        let output = self.immut_read(address);
        match address % 8 {
            2 => {
                self.status.set(Status::VBLANK, false);
                self.latch = 0;
            }
            7 => {
                self.read_buffer = self.bus.read_byte(address);
                self.increment_vram_address();
            }
            _ => (),
        }
        self.open_bus = output;
        output
    }

    pub(crate) fn write_byte(&mut self, address: u16, value: u8) {
        debug_assert!((0x2000..=0x3fff).contains(&address));
        self.open_bus = value;
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
            6 => {
                let value = value as u16;
                if self.latch == 0 {
                    self.vram_address = (self.vram_address & 0x00ff) | (value << 8);
                    self.latch = 1;
                } else {
                    self.vram_address = (self.vram_address & 0xff00) | value;
                    self.latch = 0;
                }
            }
            7 => {
                self.bus.write_byte(self.vram_address, value);
                self.increment_vram_address();
            }
            _ => unreachable!(),
        }
    }

    fn increment_vram_address(&mut self) {
        self.vram_address += if self.control.contains(Control::VRAM_INCREMENT) {
            32
        } else {
            1
        }
    }
}
