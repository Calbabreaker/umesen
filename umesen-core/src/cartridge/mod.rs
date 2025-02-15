mod cartridge_banks;
mod cartridge_header;
mod mapper000;
mod mapper220;

use crate::{
    cartridge::{mapper000::Mapper000, mapper220::Mapper220},
    NesParseError,
};
pub use cartridge_banks::{CartridgeBanks, FixedArray, MemoryBankExt};
pub use cartridge_header::{CartridgeHeader, Mirroring};

/// Generic trait for underlying circuitry inside a catridge that will read and write to a catridge memory bank
pub trait Mapper {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8>;
    fn cpu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8);
    fn ppu_read(&self, banks: &CartridgeBanks, address: u16) -> u8;
    fn ppu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8);
    /// Option to override mirroring from header
    fn mirroring(&self) -> Option<Mirroring> {
        None
    }
}

pub struct Cartridge {
    banks: CartridgeBanks,
    header: CartridgeHeader,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn from_nes(mut bytes: impl std::io::Read) -> Result<Self, NesParseError> {
        let mut header_data = [0; 16];
        bytes.read_exact(&mut header_data)?;
        let header = CartridgeHeader::from_nes(header_data)?;

        if header.has_trainer {
            let mut trainer_data = [0; 512];
            bytes.read_exact(&mut trainer_data)?;
        }

        let mut prg_rom = vec![0; header.prg_rom_size];
        bytes.read_exact(&mut prg_rom)?;
        let mut chr_rom = vec![0; header.chr_rom_size];
        bytes.read_exact(&mut chr_rom)?;

        let banks = CartridgeBanks::new(&header, prg_rom, chr_rom);
        Self::new(header, banks)
    }

    pub fn new(header: CartridgeHeader, banks: CartridgeBanks) -> Result<Self, NesParseError> {
        Ok(Cartridge {
            mapper: match header.mapper_id {
                0 => Box::new(Mapper000::default()),
                220 => Box::new(Mapper220::default()),
                id => return Err(NesParseError::UnsupportedMapper(id)),
            },
            header,
            banks,
        })
    }

    pub fn with_rom(
        mapper_id: u8,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
        prg_ram_size: usize,
    ) -> Self {
        let header = CartridgeHeader {
            mapper_id,
            prg_rom_size: prg_rom.len(),
            chr_rom_size: chr_rom.len(),
            prg_ram_size,
            ..Default::default()
        };
        let banks = CartridgeBanks::new(&header, prg_rom, chr_rom);
        Self::new(header, banks).unwrap()
    }

    pub fn cpu_read(&self, address: u16) -> Option<u8> {
        self.mapper.cpu_read(&self.banks, address)
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        self.mapper.cpu_write(&mut self.banks, address, value);
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        self.mapper.ppu_read(&self.banks, address)
    }

    pub fn ppu_write(&mut self, address: u16, value: u8) {
        self.mapper.ppu_write(&mut self.banks, address, value);
    }

    pub fn mirroring(&self) -> Mirroring {
        if let Some(mirroring) = self.mapper.mirroring() {
            mirroring
        } else {
            self.header.mirroring
        }
    }

    pub fn header(&self) -> &CartridgeHeader {
        &self.header
    }
}
