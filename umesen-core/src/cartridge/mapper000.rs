use crate::cartridge::{CartridgeBoard, CartridgeData};

/// INES designation for NROM boards
/// https://www.nesdev.org/wiki/NROM
pub struct Mapper000 {
    data: CartridgeData,
}

impl Mapper000 {
    pub fn new(data: CartridgeData) -> Self {
        Self { data }
    }
}

impl CartridgeBoard for Mapper000 {
    fn prg_read(&self, address: u16) -> u8 {
        todo!()
    }

    fn prg_write(&mut self, address: u16) {
        todo!()
    }

    fn chr_read(&self, address: u16) -> u8 {
        todo!()
    }

    fn chr_write(&mut self, address: u16) {
        todo!()
    }
}
