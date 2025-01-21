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

/// Generic trait for underlying circuitry inside a catridge
pub trait Mapper {
    fn cpu_read(&self, address: u16) -> u8;
    fn cpu_write(&mut self, address: u16, value: u8);
    fn ppu_read(&self, address: u16) -> u8;
    fn ppu_write(&mut self, address: u16, value: u8);
}

/// Wraps the Mapper trait
#[derive(Clone)]
pub struct Catridge(Rc<RefCell<dyn Mapper>>);

impl Catridge {
    pub fn from_nes(mut bytes: impl std::io::Read) -> Result<Self, NesParseError> {
        let mut header_data = [0; 16];
        read_bytes(&mut bytes, &mut header_data, 16)?;
        let header = CartridgeHeader::from_nes(header_data)?;

        if header.has_trainer {
            let mut trainer_data = [0; 512];
            read_bytes(&mut bytes, &mut trainer_data, header.total_size())?;
        }

        let mut prg_rom = vec![0; header.prg_rom_size];
        read_bytes(&mut bytes, &mut prg_rom, header.total_size())?;
        let mut chr_rom = vec![0; header.chr_rom_size];
        read_bytes(&mut bytes, &mut chr_rom, header.total_size())?;

        Self::from_data(CartridgeData::new(header, prg_rom, chr_rom))
    }

    pub fn from_data(data: CartridgeData) -> Result<Self, NesParseError> {
        Ok(Catridge(match data.header.mapper_id {
            0 => Rc::new(RefCell::new(Mapper000::new(data))),
            220 => Rc::new(RefCell::new(Mapper220::new(data))),
            _ => return Err(NesParseError::UnsupportedMapper(data.header.mapper_id)),
        }))
    }

    /// New catridge with only prg_ram (for testing)
    pub fn new_only_ram(ram_size: usize) -> Self {
        Self::new(220, vec![], vec![], ram_size)
    }

    pub fn new(mapper_id: u8, prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_ram_size: usize) -> Self {
        let header = CartridgeHeader {
            mapper_id,
            prg_rom_size: prg_rom.len(),
            chr_rom_size: chr_rom.len(),
            prg_ram_size,
            ..Default::default()
        };
        Self::from_data(CartridgeData::new(header, prg_rom, chr_rom)).unwrap()
    }

    pub fn cpu_read(&self, address: u16) -> u8 {
        debug_assert!((0x4020..=0xffff).contains(&address)); // Sanity check
        self.0.borrow().cpu_read(address)
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        debug_assert!((0x4020..=0xffff).contains(&address));
        self.0.borrow_mut().cpu_write(address, value);
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        debug_assert!((0x0000..=0x1fff).contains(&address));
        self.0.borrow().ppu_read(address)
    }

    pub fn ppu_write(&mut self, address: u16, value: u8) {
        debug_assert!((0x0000..=0x1fff).contains(&address));
        self.0.borrow_mut().ppu_write(address, value);
    }
}

fn read_bytes(
    data: &mut impl std::io::Read,
    buffer: &mut [u8],
    total_size: usize,
) -> Result<(), NesParseError> {
    if !buffer.is_empty() {
        data.read_exact(buffer)
            .map_err(|_| NesParseError::NotEnough(total_size))?;
    }
    Ok(())
}
