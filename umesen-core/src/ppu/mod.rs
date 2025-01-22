use crate::ppu::{bus::PpuBus, palette::Palette, registers::Registers};

mod bus;
mod palette;
mod registers;

#[derive(Default)]
pub struct Ppu {
    pub registers: Registers,
    pub palette: Palette,
    pub bus: PpuBus,
}

impl Ppu {}
