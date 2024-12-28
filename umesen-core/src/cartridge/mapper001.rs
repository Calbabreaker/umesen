use crate::cartridge::{CartridgeBoard, CartridgeData};

pub struct Mapper001 {
    data: CartridgeData,
}

impl Mapper001 {
    pub fn new(data: CartridgeData) -> Self {
        Self { data }
    }
}

impl CartridgeBoard for Mapper001 {
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
