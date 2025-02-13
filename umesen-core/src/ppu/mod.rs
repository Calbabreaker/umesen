use crate::{cartridge::FixedArray, ppu::sprite::Attributes};

mod bus;
mod palette;
mod registers;
pub mod sprite;

pub use bus::*;
pub use palette::Palette;
pub use registers::*;
pub use sprite::Sprite;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

/// Emulated 2C02 NTSC PPU
#[derive(Clone, Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    pub scanline: u16,
    pub dot: u16,
    pub screen_pixels: FixedArray<u32, { WIDTH * HEIGHT }>,
    pub(crate) frame_complete: bool,
    pub(crate) require_nmi: bool,

    // Bits shifted left every render dot so leftmost bit contains low and high bit of the current pixel index in the palette
    bg_shift_bits_low: u16,
    bg_shift_bits_high: u16,
    bg_palette_id: u8,
    bg_palette_bits_low: u8,
    bg_palette_bits_high: u8,
    odd_frame: bool,

    /// Buffer of sprites to render next scanline
    sprite_buffer: [Sprite; 8],
    sprite_count: u8,
    oam_start_address: u8,
}

impl Ppu {
    /// Gets a RGBA color from a the palette ram based on index from 0-64 (8 palette with 4 indexable colors)
    pub fn get_palette_color(&self, color_index: impl Into<u16>) -> u32 {
        let offset = color_index.into();
        debug_assert!((0..64).contains(&offset));
        let palette_index = self.registers.bus.read_u8(0x3f00 + offset);
        self.palette.get(palette_index)
    }

    pub fn get_palette_colors(&self, palette_id: u8) -> [u32; 4] {
        let mut palette = [0; 4];
        for (i, color) in palette.iter_mut().enumerate() {
            *color = self.get_palette_color(palette_id * 4 + i as u8);
        }
        palette
    }

    pub(crate) fn clock(&mut self) {
        // Specific scanline timings
        // See https://www.nesdev.org/w/images/default/4/4f/Ppu.svg
        match self.scanline {
            0..=239 => {
                self.frame_complete = false;
                self.clock_sprite_render_line();
                self.clock_bg_render_line();
            }
            241 if self.dot == 1 => {
                self.frame_complete = true;
                self.registers.status.set(Status::VBLANK, true);
                if self.registers.control.contains(Control::VBLANK_NMI) {
                    self.require_nmi = true;
                }
            }
            261 => {
                self.clock_sprite_render_line();
                self.clock_bg_prerender_line();
            }
            _ => (),
        }

        // Get the pixel color and set in the screen
        if self.dot >= 1 {
            let x = (self.dot - 1) as usize;
            let y = self.scanline as usize;
            if x < WIDTH && y < HEIGHT && self.registers.mask.is_rendering() {
                let bg_color_index = self.render_bg_pixel(x);
                let fg_color_index = self.render_fg_pixel(x, bg_color_index);
                self.screen_pixels[x + y * WIDTH] = self.get_palette_color(fg_color_index);
            }
        }

        self.next_dot();
    }

    // Scanlines when the PPU is actually drawing to the screen
    fn clock_bg_render_line(&mut self) {
        match self.dot {
            // When at visible dots, shift registers and load bits
            // Last 16 dots load the shift register with the next tiles on the next scanline
            1..=256 | 328..=336 => {
                self.shift_registers();

                if (self.dot - 1) % 8 == 0 {
                    self.load_background_shift_bits();
                }
            }
            257 => {
                // Scroll y and reset x when end of visible scanline
                // Scroll y technically meant to happen previous dot but should work still
                if self.registers.mask.is_rendering() {
                    self.registers.v.scroll_fine_y();
                    self.registers.v.set_x(&self.registers.t);
                }
            }
            _ => (),
        }
    }

    fn clock_bg_prerender_line(&mut self) {
        match self.dot {
            1 => {
                self.registers.status.remove(Status::VBLANK);
                self.registers.status.remove(Status::SPRITE_OVERFLOW);
                self.registers.status.remove(Status::SPRITE_0_HIT);
            }
            280..=304 => {
                if self.registers.mask.is_rendering() {
                    self.registers.v.set_y(&self.registers.t);
                }
            }
            339 => {
                // Skip last cycle on odd frames
                if self.registers.mask.is_rendering() && self.odd_frame {
                    self.dot += 1;
                }
            }
            // Prerender line does same stuff as render line but with extra stuff
            _ => self.clock_bg_render_line(),
        }
    }

    /// Load the next tile data into the background shift bits for the current tile at the dot scanline
    /// Technically the address calculation and reads are supposed to happen in different
    /// cycles every 8 cycles but just have it all at once for simplicity (maybe slightly less accurate)
    fn load_background_shift_bits(&mut self) {
        let registers = &mut self.registers;
        let tile_number = registers.bus.read_u8(registers.v.nametable_address());
        let attribute_byte = registers.bus.read_u8(registers.v.attribute_address());
        self.bg_palette_id = registers.v.shift_attribute(attribute_byte);
        let (tile_lsb, tile_msb) = registers.bus.read_pattern_tile_planes(
            tile_number as u16 + registers.control.background_table_offset(),
            registers.v.get(TvRegister::FINE_Y),
        );

        self.bg_shift_bits_low = (self.bg_shift_bits_low & 0xff00) | tile_lsb as u16;
        self.bg_shift_bits_high = (self.bg_shift_bits_high & 0xff00) | tile_msb as u16;

        if registers.mask.is_rendering() {
            registers.v.scroll_coarse_x();
        }
    }

    fn clock_sprite_render_line(&mut self) {
        match self.dot {
            // The start location of oam evaluation is taken from oam_address on dot 65
            65 => self.oam_start_address = self.registers.oam_address,
            257 => {
                if self.scanline < HEIGHT as u16 {
                    // Technically supposed to happen for the entire scanline but do it once at the end for simplicity
                    self.eval_sprites();
                }
            }
            258..=320 => self.registers.oam_address = 0,
            321 => {
                for sprite in &mut self.sprite_buffer[0..self.sprite_count as usize] {
                    sprite.load_shift_bits(self.scanline, &self.registers);
                }
            }
            _ => (),
        }
    }

    fn eval_sprites(&mut self) {
        self.sprite_count = 0;
        let mut i = self.oam_start_address as usize;

        while i < self.registers.oam_data.len() {
            // Get four bytes but make sure to not overflow array
            let right_bound = (i + 4).min(self.registers.oam_data.len());
            let chunk = &self.registers.oam_data[i..right_bound];
            let mut sprite = Sprite::new(chunk);
            sprite.oam_index = (i / 4) as u8;

            // Add to sprite buffer if sprite part of scanline
            if sprite.y_intersects(self.scanline, self.registers.control.sprite_height()) {
                // Check sprite overflow
                if self.sprite_count == 8 {
                    self.registers.status.insert(Status::SPRITE_OVERFLOW);
                    break;
                }

                self.sprite_buffer[self.sprite_count as usize] = sprite;
                self.sprite_count += 1;
            } else if self.sprite_count == 8 {
                // After 8 sprites has been filled, PPU check for overflow by
                // searching for another sprite that is in the scanline.
                // But for some reason, when it doesn't find a sprite after full,
                // the next OAM y it checks is offseted by the oam size + 1 which causes buggy behaviour when setting SPRITE_OVERFLOW flag.
                i += 1;
            }

            i += 4;
        }
    }

    /// Returns the index into palette ram for the current pixel of the background (0 means no color/transparent)
    fn render_bg_pixel(&mut self, scan_x: usize) -> u8 {
        if !self.registers.mask.can_show_background(scan_x) {
            return 0;
        }

        let bit_mask = 0b1000_0000 >> self.registers.fine_x;
        let mut color_index = add_bit_planes(
            (self.bg_shift_bits_low >> 8) as u8,
            (self.bg_shift_bits_high >> 8) as u8,
            bit_mask,
        );

        if color_index != 0 {
            // Add palette offset if not transparent
            let palette_id = add_bit_planes(
                self.bg_palette_bits_low,
                self.bg_palette_bits_high,
                bit_mask,
            );
            color_index += 4 * palette_id
        }

        color_index
    }

    /// Returns the palette ram index for the current pixel if a sprite is there or the background based on piority
    fn render_fg_pixel(&mut self, scan_x: usize, bg_color_index: u8) -> u8 {
        let mut fg_color_index = bg_color_index;

        if self.registers.mask.can_show_sprite(scan_x) {
            // Find sprite to render
            for sprite in &self.sprite_buffer[0..self.sprite_count as usize] {
                let color_index = sprite.color_index(scan_x);
                if color_index == 0 {
                    continue;
                }

                // Check if sprite 0 is rendering and set status flag
                if sprite.oam_index == 0 && bg_color_index != 0 && scan_x != 255 {
                    self.registers.status.insert(Status::SPRITE_0_HIT);
                }

                let palette_id = sprite.attributes.palette() + 4;
                let behind_bg = sprite.attributes.contains(Attributes::BEHIND);
                // Set the pallete ram index if over background or background is transparent
                if !behind_bg || bg_color_index == 0 {
                    fg_color_index = color_index + palette_id * 4;
                }
            }
        }

        fg_color_index
    }

    fn shift_registers(&mut self) {
        self.bg_shift_bits_low <<= 1;
        self.bg_shift_bits_high <<= 1;
        // Shift and add new bits to the shift register
        let palette_lsb = self.bg_palette_id & 0b01;
        let palette_msb = (self.bg_palette_id & 0b10) >> 1;
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
            self.odd_frame = !self.odd_frame;
            self.scanline = 0;
        }
    }
}

/// Adds two low and high bits specified by the bit mask
pub fn add_bit_planes(lsb_plane: u8, msb_plane: u8, bit_mask: u8) -> u8 {
    let lsb = ((lsb_plane & bit_mask) != 0) as u8;
    let msb = ((msb_plane & bit_mask) != 0) as u8;
    lsb | (msb << 1)
}
