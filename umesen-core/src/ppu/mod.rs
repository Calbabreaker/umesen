use crate::cartridge::FixedArray;

mod bus;
mod palette;
mod registers;

pub use bus::*;
pub use palette::Palette;
pub use registers::*;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;
pub const FRAME_INTERVAL: f64 = 1. / 60.;

/// Emulated 2C02 NTSC PPU
#[derive(Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    pub scanline: u16,
    pub dot: u16,
    pub screen_pixels: FixedArray<u32, { WIDTH * HEIGHT }>,
    pub(crate) frame_complete: bool,
    pub(crate) require_nmi: bool,

    bg_palette_id: u8,
    bg_shift_bits_low: u16,
    bg_shift_bits_high: u16,
    bg_palette_bits_low: u8,
    bg_palette_bits_high: u8,
}

impl Ppu {
    pub(crate) fn clock(&mut self) {
        let x = self.dot as usize + 1;
        let y = self.scanline as usize;
        if x < WIDTH && y < HEIGHT {
            self.render_pixel(x, y);
        }

        // See https://www.nesdev.org/w/images/default/4/4f/Ppu.svg
        match self.scanline {
            0..=239 => self.clock_render_line(),
            241 if self.dot == 1 => {
                self.registers.status.set(Status::VBLANK, true);
                if self.registers.control.contains(Control::VBLANK_NMI) {
                    self.require_nmi = true;
                }
            }
            261 => self.clock_prerender_line(),
            _ => (),
        }

        self.next_dot();
    }

    /// Gets a RGBA color from a palette id with a 0-3 pixel offset
    pub fn get_palette_color(&self, palette_id: u8, i: u8) -> u32 {
        debug_assert!((0..=4).contains(&i));
        debug_assert!((0..=7).contains(&palette_id));
        let offset = (palette_id * 4 + i) as u16;
        let palette_index = self.registers.bus.read_u8(0x3f00 + offset);
        self.palette.get(palette_index)
    }

    // Scanlines when the PPU is actually drawing to the screen
    fn clock_render_line(&mut self) {
        match self.dot {
            1..=256 | 336 => {
                if (self.dot - 1) % 8 == 7 {
                    self.load_background_shift_bits();
                }
            }
            257 => {
                if self.registers.mask.is_rendering() {
                    self.registers.v.scroll_fine_y();
                    self.registers.v.set_x(&self.registers.t);
                }
            }
            _ => (),
        }
    }

    fn clock_prerender_line(&mut self) {
        match self.dot {
            1 => {
                self.registers.status.set(Status::VBLANK, false);
            }
            304 => {
                if self.registers.mask.is_rendering() {
                    self.registers.v.set_y(&self.registers.t);
                }
            }
            1..=256 | 336 => {
                self.clock_render_line();
            }
            _ => (),
        }
    }

    /// Load the next tile data into the background shift bits for the current tile at the dot scanline
    /// Technically the address calculation and reads are supposed to happen in different
    /// cycles every 8 cycles but just have it all here for simplicity
    fn load_background_shift_bits(&mut self) {
        let registers = &mut self.registers;
        let tile_number = registers.bus.read_u8(registers.v.nametable_address());
        let attribute_byte = registers.bus.read_u8(registers.v.attribute_address());
        self.bg_palette_id = registers.v.shift_attribute(attribute_byte);
        let (tile_lsb, tile_msb) = registers.bus.read_pattern_tile_planes(
            tile_number,
            registers.control.contains(Control::BACKGROUND_TABLE_OFFSET) as u8,
            registers.v.get(TvRegister::FINE_Y) as u8,
        );

        self.bg_shift_bits_low = (self.bg_shift_bits_low & 0xff00) | tile_lsb as u16;
        self.bg_shift_bits_high = (self.bg_shift_bits_low & 0xff00) | tile_msb as u16;

        if self.registers.mask.is_rendering() {
            self.registers.v.scroll_coarse_x();
        }
    }

    fn render_pixel(&mut self, x: usize, y: usize) {
        if !self.registers.mask.contains(Mask::RENDER_BACKGROUND) {
            return;
        }

        let bit_mask = 0b1000_0000 >> self.registers.fine_x;
        let pixel_index = add_bit_planes(
            self.bg_shift_bits_low as u8,
            self.bg_shift_bits_high as u8,
            bit_mask,
        );

        let palette_id = add_bit_planes(
            self.bg_palette_bits_low,
            self.bg_palette_bits_high,
            bit_mask,
        );

        let color = self.get_palette_color(palette_id, pixel_index);
        self.screen_pixels[x + y * WIDTH] = color;

        self.shift_registers();
    }

    fn shift_registers(&mut self) {
        let palette_lsb = self.bg_palette_id & 0b01;
        let palette_msb = (self.bg_palette_id & 0b10) >> 1;
        self.bg_shift_bits_low <<= 1;
        self.bg_shift_bits_high <<= 1;
        // Shift and add new bits to the shift register
        self.bg_palette_bits_low = (self.bg_palette_bits_low << 1) | palette_lsb;
        self.bg_palette_bits_high = (self.bg_palette_bits_high << 1) | palette_msb;
    }

    fn next_dot(&mut self) {
        self.dot += 1;
        if self.dot == 341 {
            self.dot = 0;
            self.scanline += 1;
        }

        if self.scanline == 262 {
            self.frame_complete = true;
            self.scanline = 0;
        }
    }
}

pub fn add_bit_planes(lsb_plane: u8, msb_plane: u8, bit_mask: u8) -> u8 {
    let lsb = ((lsb_plane & bit_mask) != 0) as u8;
    let msb = ((msb_plane & bit_mask) != 0) as u8;
    lsb | (msb << 1)
}
