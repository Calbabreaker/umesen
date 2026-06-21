use crate::{cartridge::FixedArray, ppu::sprite::Attributes};

mod bus;
mod palette;
mod registers;
pub mod sprite;
mod vram;

pub use bus::*;
pub use palette::Palette;
pub use registers::*;
pub use sprite::Sprite;
pub use vram::VramRegister;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;
pub const MAX_SPRITES_PER_SCAN: usize = 8;
pub const PRERENDER_SCANLINE: usize = 261;

pub type ScreenPixels = FixedArray<FixedArray<u8, 3>, { WIDTH * HEIGHT }>;

pub enum PpuClockReport {
    None,
    Nmi,
    EndScanline,
}

/// Emulated 2C02 NTSC PPU
#[derive(Clone, Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    pub screen_pixels: ScreenPixels,
    pub unlimited_sprites: bool,
    frame_complete: bool,

    // Bits shifted left every render dot so leftmost bit contains low and high bit of the current pixel index in the palette
    bg_shift_bits_low: u16,
    bg_shift_bits_high: u16,
    bg_palette_id: u8,
    bg_palette_bits_low: u8,
    bg_palette_bits_high: u8,

    /// Buffer of sprites to render next scanline
    sprite_buffer: Vec<Sprite>,
    oam_start_address: u8,
}

impl Ppu {
    /// Gets a RGB color from the palette ram based on index from 0-64 (8 palettes with 4 indexable colors)
    pub fn get_palette_color(&self, color_index: impl Into<u16>) -> [u8; 3] {
        let offset = color_index.into();
        std::debug_assert_matches!(offset, 0..64);
        self.palette.get(self.registers.read_palette_ram(offset))
    }

    pub fn get_palette_colors(&self, palette_id: u8) -> [[u8; 3]; 4] {
        let mut palette = [[0; 3]; 4];
        for (i, color) in palette.iter_mut().enumerate() {
            *color = self.get_palette_color(palette_id * 4 + i as u8);
        }
        palette
    }

    pub fn frame_complete(&mut self) -> bool {
        if self.frame_complete {
            self.frame_complete = false;
            true
        } else {
            false
        }
    }

    pub(crate) fn clock(&mut self) -> PpuClockReport {
        let mut report = PpuClockReport::None;

        // Specific scanline timings
        // See https://www.nesdev.org/w/images/default/4/4f/Ppu.svg
        match self.registers.scanline {
            0..=239 => {
                self.frame_complete = false;
                self.clock_sprite_render_line();
                self.clock_bg_render_line();
                if self.registers.on_visble_dot() {
                    self.render_pixel();
                }
                report = self.check_scanline_report();
            }
            241 if self.registers.dot == 1 => {
                self.frame_complete = true;
                self.registers.status.set(Status::VBLANK, true);
                if self.registers.control.contains(Control::VBLANK_NMI) {
                    report = PpuClockReport::Nmi
                }
            }
            PRERENDER_SCANLINE => {
                self.clock_sprite_render_line();
                self.clock_bg_prerender_line();
                report = self.check_scanline_report();
            }
            _ => (),
        }

        self.registers.next_dot();
        report
    }

    fn check_scanline_report(&self) -> PpuClockReport {
        if self.registers.dot == 260 && self.registers.mask.is_rendering() {
            PpuClockReport::EndScanline
        } else {
            PpuClockReport::None
        }
    }

    fn render_pixel(&mut self) {
        let x = self.registers.dot - 1;
        let color_index = if self.registers.mask.is_rendering() {
            let bg_color_index = self.render_bg_pixel(x);
            self.render_fg_pixel(x, bg_color_index)
        } else if matches!(self.registers.v.0, PALETTE_START..=0x3fff) {
            // If v register is pointing to pallette address then use that color instead
            // of the backdrop color
            (self.registers.v.0 - PALETTE_START) as u8
        } else {
            0
        };
        *self.screen_pixels[x + self.registers.scanline * WIDTH] =
            self.get_palette_color(color_index);
    }

    // Scanlines when the PPU is actually drawing to the screen
    fn clock_bg_render_line(&mut self) {
        match self.registers.dot {
            // When at visible dots, shift registers and load bits
            // Last 16 dots load the shift register with the next tiles on the next scanline
            1..=256 | 328..=336 => {
                self.shift_registers();

                if (self.registers.dot - 1).is_multiple_of(8) {
                    self.load_background_shift_bits();
                }
            }
            // Scroll y and reset x when end of visible scanline
            // Scroll y technically meant to happen previous dot but should work still
            257 if self.registers.mask.is_rendering() => {
                self.registers.v.scroll_fine_y();
                self.registers.v.set_x(&self.registers.t);
            }
            _ => (),
        }
    }

    fn clock_bg_prerender_line(&mut self) {
        match self.registers.dot {
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
                if self.registers.mask.is_rendering() && self.registers.frame_count % 2 == 1 {
                    self.registers.dot += 1;
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
        self.bg_palette_id = registers.v.palette_id(attribute_byte);
        let (tile_lsb, tile_msb) = registers.bus.read_pattern_tile_planes(
            tile_number as u16 + registers.control.background_table_offset(),
            registers.v.get(VramRegister::FINE_Y),
        );

        self.bg_shift_bits_low = (self.bg_shift_bits_low & 0xff00) | tile_lsb as u16;
        self.bg_shift_bits_high = (self.bg_shift_bits_high & 0xff00) | tile_msb as u16;

        if registers.mask.is_rendering() {
            registers.v.scroll_coarse_x();
        }
    }

    fn clock_sprite_render_line(&mut self) {
        match self.registers.dot {
            // The start location of oam evaluation is taken from oam_address on dot 65
            65 => self.oam_start_address = self.registers.oam_address,
            // Technically supposed to happen for the entire scanline but do it once at the end for simplicity
            256 => self.sprite_buffer.clear(),
            257 if self.registers.scanline < HEIGHT => self.eval_sprites(),
            258..=320 => self.registers.oam_address = 0,
            321 => {
                for sprite in &mut self.sprite_buffer {
                    sprite.load_shift_bits(self.registers.scanline as u16, &self.registers);
                }
            }
            _ => (),
        }
    }

    /// Populates the sprite buffer for the next scanline and checks SPRITE_OVERFLOW
    fn eval_sprites(&mut self) {
        let mut i = 0;
        let mut byte_offset = self.oam_start_address as usize;
        let height = self.registers.control.sprite_height() as usize;
        while let Some(sprite) = self.registers.get_oam_sprite(i, byte_offset) {
            // Add to sprite buffer if sprite part of scanline
            // Note that it's loading sprites for the next scanline so all sprite y is offset by one
            if self.registers.scanline >= sprite.y as usize
                && self.registers.scanline < sprite.y as usize + height
            {
                // Check sprite overflow
                if self.sprite_buffer.len() == MAX_SPRITES_PER_SCAN && !self.unlimited_sprites {
                    self.registers.status.insert(Status::SPRITE_OVERFLOW);
                    break;
                }

                self.sprite_buffer.push(sprite);
            } else if self.sprite_buffer.len() == MAX_SPRITES_PER_SCAN && !self.unlimited_sprites {
                // After 8 sprites has been filled, the PPU will check for overflow by
                // searching for another sprite that is in the scanline.
                // But for some reason, when it doesn't find a sprite after filled,
                // the next OAM y it checks is offseted by one extra byte which causes buggy
                // behaviour when setting SPRITE_OVERFLOW flag.
                byte_offset += 1;
            }

            i += 1;
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
        if self.registers.mask.can_show_sprite(scan_x) {
            // Find sprite to render
            for sprite in &self.sprite_buffer {
                let color_index = sprite.color_index(scan_x);
                if color_index == 0 {
                    continue;
                }

                // Check if sprite 0 is rendering and set status flag
                if sprite.oam_index == 0 && bg_color_index != 0 && scan_x != 255 {
                    self.registers.status.insert(Status::SPRITE_0_HIT);
                }

                let palette_id = sprite.attributes.palette() + 4;
                let behind_bg = sprite.attributes.contains(Attributes::RENDER_BEHIND);
                if behind_bg && bg_color_index != 0 {
                    // Sprite still gets drawn on top of later sprites but it uses background
                    return bg_color_index;
                } else {
                    return color_index + palette_id * 4;
                }
            }
        }

        bg_color_index
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
}

/// Adds two low and high bits specified by the bit mask
pub fn add_bit_planes(lsb_plane: u8, msb_plane: u8, bit_mask: u8) -> u8 {
    let lsb = ((lsb_plane & bit_mask) != 0) as u8;
    let msb = ((msb_plane & bit_mask) != 0) as u8;
    lsb | (msb << 1)
}
