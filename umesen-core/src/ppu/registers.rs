use crate::ppu::bus::PpuBus;

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Control: u8 {
        /// X bit of nametable
        const NAMETABLE_X = 1;
        /// Y bit of nametable
        const NAMETABLE_Y = 1 << 1;
        /// 0: add 1, 1: add 32
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
    pub const SELECT_COARSE_X:    u16 = 0b00000000_00011111;
    pub const SELECT_COARSE_Y:    u16 = 0b00000011_11100000;
    pub const SELECT_NAMETABLE_X: u16 = 0b00000100_00000000;
    pub const SELECT_NAMETABLE_Y: u16 = 0b00001000_00000000;
    pub const SELECT_FINE_Y:      u16 = 0b01110000_00000000;
    pub const SELECT_LOW:         u16 = 0b00000000_11111111;
    pub const SELECT_HIGH:        u16 = 0b11111111_00000000;
}

impl TvRegister {
    pub fn set(&mut self, value: impl Into<u16>, select_bits: u16) {
        let value = value.into();
        let value_shifted = value << select_bits.trailing_zeros();
        self.0 = value_shifted | (self.0 & (!select_bits));
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
    pub t_register: TvRegister,
    pub v_register: TvRegister,
    pub latch: bool,
    oam_address: u8,
    oam_data: u8,
    read_buffer: u8,
    open_bus: u8,
    fine_x: u8,
}

impl Registers {
    pub(crate) fn immut_read_u8(&self, address: u16) -> u8 {
        debug_assert!((0x2000..=0x3fff).contains(&address));
        match address % 8 {
            // Fill the unused bits with open bus
            2 => self.status.bits() | (self.open_bus & (!Status::all().bits())),
            4 => self.oam_data,
            7 => {
                // Palette address gets data returned immediately instead of being buffered
                if self.v_register.0 >= 0x3f00 {
                    self.bus.read_u8(self.v_register.0)
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
                self.status.set(Status::VBLANK, false);
                self.latch = false;
            }
            7 => {
                self.read_buffer = self.bus.read_u8(self.v_register.0);
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
            0 => {
                self.control = Control::from_bits(value).unwrap();
                let nametable_x = self.control.contains(Control::NAMETABLE_X);
                let nametable_y = self.control.contains(Control::NAMETABLE_Y);
                self.t_register
                    .set(nametable_x, TvRegister::SELECT_NAMETABLE_X);
                self.t_register
                    .set(nametable_y, TvRegister::SELECT_NAMETABLE_Y);
            }
            1 => self.mask = Mask::from_bits(value).unwrap(),
            2 => (),
            3 => self.oam_address = value,
            4 => {
                self.oam_data = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            // Scroll write
            5 => {
                let fine = value & 0b111;
                let coarse = value >> 3;
                if !self.latch {
                    // X scroll
                    self.t_register.set(coarse, TvRegister::SELECT_COARSE_X);
                    self.fine_x = fine;
                } else {
                    // Y Scroll
                    self.t_register.set(coarse, TvRegister::SELECT_COARSE_Y);
                    self.t_register.set(fine, TvRegister::SELECT_FINE_Y);
                }
                self.latch = !self.latch;
            }
            // VRAM address write
            6 => {
                if !self.latch {
                    // Write the high byte to t_register with the last bit unset
                    self.t_register.set(value & 0x3f, TvRegister::SELECT_HIGH);
                } else {
                    // Write low byte and copy to v_register
                    self.t_register.set(value, TvRegister::SELECT_LOW);
                    // println!("{:x}", self.t_register.0);
                    self.v_register = self.t_register;
                }
                self.latch = !self.latch;
            }
            // VRAM data write
            7 => {
                self.bus.write_u8(self.v_register.0, value);
                self.increment_v_register();
            }
            _ => unreachable!(),
        }
    }

    fn increment_v_register(&mut self) {
        self.v_register.0 += if self.control.contains(Control::VRAM_INCREMENT) {
            32
        } else {
            1
        };
    }
}
