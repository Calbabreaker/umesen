use crate::ppu::registers::Registers;

mod registers;

#[derive(Default)]
pub struct Ppu {
    pub registers: Registers,
}

impl Ppu {}
