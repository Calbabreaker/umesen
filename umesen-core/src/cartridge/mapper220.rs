use crate::cartridge::{CartridgeData, Mapper};

/// Mapper is not assigned by INES to anything useful so this will be used as a mapper for testing
/// This is just going to have ram
pub struct Mapper220 {
    data: CartridgeData,
}

impl Mapper220 {
    pub fn new(data: CartridgeData) -> Self {
        Self { data }
    }
}

impl Mapper for Mapper220 {
    fn cpu_read(&self, address: u16) -> u8 {
        self.data.prg_ram.mirrored_read(address)
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        self.data.prg_ram.mirrored_write(address, value)
    }

    fn ppu_read(&self, _: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, _: u16, _: u8) {}
}
