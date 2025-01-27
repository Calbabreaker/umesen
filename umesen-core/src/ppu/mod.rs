use crate::cartridge::FixedArray;

mod bus;
mod palette;
mod registers;

pub use bus::PpuBus;
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
    pub cycle: u16,
    pub screen_pixels: FixedArray<u32, { WIDTH * HEIGHT }>,
    pub(crate) frame_complete: bool,
    pub(crate) require_nmi: bool,

    address_to_read: u16,
    background_tile_number: u8,
    background_tile_attribute: u8,
}

impl Ppu {
    pub(crate) fn clock(&mut self) {
        // See https://www.nesdev.org/w/images/default/4/4f/Ppu.svg
        match self.scanline {
            // Scanlines when the PPU is actually drawing to the screen
            0..=239 => self.clock_drawing_line(),
            261 => self.clock_prerender_line(),
            241 => {
                if self.cycle == 1 {
                    self.registers.status.set(Status::VBLANK, true);
                    if self.registers.control.contains(Control::VBLANK_NMI) {
                        self.require_nmi = true;
                    }
                }
            }
            _ => (),
        }

        let x = self.cycle as usize + 1;
        let y = self.scanline as usize;
        if x < WIDTH && y < HEIGHT {
            self.screen_pixels[x + y * WIDTH] = (x as u32).wrapping_pow(y as u32);
        }

        self.next_cycle();
    }

    /// Gets a RGBA color from a palette id with a 0-3 pixel offset
    pub fn get_palette_color(&self, palette_id: u8, i: u8) -> u32 {
        debug_assert!((0..=4).contains(&i));
        debug_assert!((0..=7).contains(&palette_id));
        let offset = (palette_id * 4 + i) as u16;
        let palette_index = self.registers.bus.read_u8(0x3f00 + offset);
        self.palette.get(palette_index)
    }

    fn clock_drawing_line(&mut self) {
        match self.cycle {
            1..=256 | 321..=336 => self.clock_drawing_cycle(self.cycle - 1),
            _ => (),
        }
    }

    /// Clock set of 8 repeating cycles to fetch the tile to draw
    fn clock_drawing_cycle(&mut self, cycle: u16) {
        match cycle % 8 {
            0 => {
                self.address_to_read = self.registers.t_register.nametable_address();
            }
            1 => {
                self.background_tile_number = self.registers.bus.read_u8(self.address_to_read);
            }
            2 => {
                self.address_to_read = self.registers.t_register.attribute_address();
            }
            3 => {
                self.background_tile_attribute = self.registers.bus.read_u8(self.address_to_read);
            }
            4 => {}
            6 => {}
            7 => {}
            _ => (),
        }
    }

    fn clock_prerender_line(&mut self) {
        match self.cycle {
            1 => {
                self.registers.status.set(Status::VBLANK, false);
            }
            _ => (),
        }
    }

    fn next_cycle(&mut self) {
        self.cycle += 1;
        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
        }

        if self.scanline == 262 {
            self.frame_complete = true;
            self.scanline = 0;
        }
    }
}
