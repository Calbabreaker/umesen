mod cartridge_data;
mod cartridge_header;
mod mapper000;
mod mapper220;

use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::{mapper000::Mapper000, mapper220::Mapper220},
    NesParseError,
};
pub use cartridge_data::CartridgeData;
pub use cartridge_header::CartridgeHeader;

pub trait CartridgeBoard {
    // R/W access for cpu bus line
    fn prg_read(&self, address: u16) -> u8;
    fn prg_write(&mut self, address: u16, value: u8);
    // R/W access for PPU bus line
    fn chr_read(&self, address: u16) -> u8;
    fn chr_write(&mut self, address: u16, value: u8);
}

pub struct Cartridge;

impl Cartridge {
    pub fn from_nes(
        mut data: impl std::io::Read,
    ) -> Result<Rc<RefCell<dyn CartridgeBoard>>, NesParseError> {
        let mut header_data = [0; 16];
        data.read_exact(&mut header_data)?;
        let header = CartridgeHeader::from_nes(header_data)?;
        if header.has_trainer {
            let mut trainer_data = [0; 512];
            data.read_exact(&mut trainer_data)?;
        }

        let mut prg_rom = vec![0; header.prg_rom_size];
        data.read_exact(&mut prg_rom)?;

        let mut chr_rom = vec![0; header.chr_rom_size];
        if !chr_rom.is_empty() {
            data.read_exact(&mut chr_rom)?;
        }

        Self::new_board(CartridgeData::new(header, prg_rom, chr_rom))
    }

    pub fn new_board(
        data: CartridgeData,
    ) -> Result<Rc<RefCell<dyn CartridgeBoard>>, NesParseError> {
        Ok(match data.header.mapper_id {
            0 => Rc::new(RefCell::new(Mapper000::new(data))),
            220 => Rc::new(RefCell::new(Mapper220::new(data))),
            _ => return Err(NesParseError::UnsupportedMapper(data.header.mapper_id)),
        })
    }

    pub fn new_only_ram(ram_size: usize) -> Rc<RefCell<dyn CartridgeBoard>> {
        Cartridge::new_board(CartridgeData::new(
            CartridgeHeader {
                mapper_id: 220,
                prg_ram_size: ram_size,
                ..Default::default()
            },
            vec![],
            vec![],
        ))
        .unwrap()
    }
}