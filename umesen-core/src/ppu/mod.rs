use crate::ppu::{bus::PpuBus, palette::Palette, registers::Registers};

mod bus;
mod palette;
mod registers;

/// Emulated 2C02 PPU
#[derive(Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    pub bus: PpuBus,
}

impl Ppu {
    pub fn clock(&mut self) {}

    pub fn get_palette_color(&self, offset: u16) -> u32 {
        let palette_index = self.bus.read_byte(0x3f00 + offset);
        self.palette.get(palette_index)
    }
}
