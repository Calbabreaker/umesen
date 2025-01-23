use crate::{
    cartridge::FixedArray,
    ppu::{bus::PpuBus, palette::Palette, registers::Registers},
};

mod bus;
mod palette;
mod registers;

pub const WIDTH: usize = 240;
pub const HEIGHT: usize = 256;
pub const FRAME_TIME: f64 = 1. / 60.;

/// Emulated 2C02 PPU
#[derive(Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    pub bus: PpuBus,
    scanline: u16,
    cycle: u16,
    pub screen_pixels: FixedArray<u32, { WIDTH * HEIGHT }>,
}

impl Ppu {
    pub fn clock(&mut self) {
        self.cycle += 1;
        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
        }

        if self.scanline == 262 {
            self.scanline = 0;
        }
    }

    pub fn frame_complete(&self) -> bool {
        self.scanline == 0
    }

    pub fn get_palette_color(&self, offset: u16) -> u32 {
        let palette_index = self.bus.read_byte(0x3f00 + offset);
        self.palette.get(palette_index)
    }
}
