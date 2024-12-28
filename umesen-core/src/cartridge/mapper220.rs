use crate::cartridge::{CartridgeBoard, CartridgeData};

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

impl CartridgeBoard for Mapper220 {
    fn prg_read(&self, address: u16) -> u8 {
        self.data.prg_ram.mirrored_read(0x4000, address)
    }

    fn prg_write(&mut self, address: u16, value: u8) {
        self.data.prg_ram.mirrored_write(0x4000, address, value)
    }

    fn chr_read(&self, address: u16) -> u8 {
        todo!()
    }

    fn chr_write(&mut self, address: u16, value: u8) {
        todo!()
    }
}
