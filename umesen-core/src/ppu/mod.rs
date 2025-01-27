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

/// Emulated 2C02 PPU
#[derive(Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    pub scanline: u16,
    pub cycle: u16,
    pub screen_pixels: FixedArray<u32, { WIDTH * HEIGHT }>,
    pub(crate) frame_complete: bool,
    pub(crate) require_nmi: bool,
}

impl Ppu {
    pub(crate) fn clock(&mut self) {
        if self.cycle == 1 {
            if self.scanline == 241 {
                self.registers.status.set(Status::VBLANK, true);
                if self.registers.control.contains(Control::VBLANK_NMI) {
                    self.require_nmi = true;
                }
            } else if self.scanline == 261 {
                self.registers.status.set(Status::VBLANK, false);
            }
        }

        let x = self.cycle as usize + 1;
        let y = self.scanline as usize;
        if x < WIDTH && y < HEIGHT {
            self.screen_pixels[x + y * WIDTH] = (x.wrapping_pow(y as u32)) as u32;
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
