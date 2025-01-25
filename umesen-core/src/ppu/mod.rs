use crate::{
    cartridge::FixedArray,
    ppu::registers::{Control, Status},
};

mod bus;
mod palette;
mod registers;

pub use bus::PpuBus;
pub use palette::Palette;
pub use registers::Registers;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;
pub const FRAME_INTERVAL: f64 = 1. / 60.;

/// Emulated 2C02 PPU
#[derive(Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    scanline: u16,
    cycle: u16,
    pub screen_pixels: FixedArray<u32, { WIDTH * HEIGHT }>,
    pub frame_complete: bool,
    pub(crate) require_nmi: bool,
}

impl Ppu {
    pub fn clock(&mut self) {
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

    pub fn get_palette_color(&self, offset: u16) -> u32 {
        let palette_index = self.registers.bus.read_byte(0x3f00 + offset);
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
