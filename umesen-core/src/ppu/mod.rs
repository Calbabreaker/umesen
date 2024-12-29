use crate::ppu::registers::PpuRegisters;

mod registers;

#[derive(Default)]
pub struct Ppu {
    pub registers: PpuRegisters,
}

impl Ppu {}
