mod cartridge_data;
mod cartridge_header;
mod mapper000;
mod mapper001;

use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::{mapper000::Mapper000, mapper001::Mapper001},
    NesParseError,
};

pub use cartridge_data::CartridgeData;
pub use cartridge_header::CartridgeHeader;

pub trait CartridgeBoard {
    // R/W access for cpu bus line
    fn prg_read(&self, address: u16) -> u8;
    fn prg_write(&mut self, address: u16);
    // R/W access for PPU bus line
    fn chr_read(&self, address: u16) -> u8;
    fn chr_write(&mut self, address: u16);
}

pub struct Cartridge;

impl Cartridge {
    pub fn from_nes(
        mut data: impl std::io::Read,
    ) -> Result<Rc<RefCell<dyn CartridgeBoard>>, NesParseError> {
        let mut header_data = [0; 16];
        data.read_exact(&mut header_data)?;
        let header = CartridgeHeader::from_nes(header_data)?;
        let data = CartridgeData::from_nes(header, data)?;
        Self::new(data)
    }

    pub fn new(data: CartridgeData) -> Result<Rc<RefCell<dyn CartridgeBoard>>, NesParseError> {
        Ok(match data.header.mapper_id {
            0 => Rc::new(RefCell::new(Mapper000::new(data))),
            1 => Rc::new(RefCell::new(Mapper001::new(data))),
            _ => return Err(NesParseError::UnsupportedMapper(data.header.mapper_id)),
        })
    }
}
