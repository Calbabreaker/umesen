mod cartridge_banks;
mod cartridge_header;
mod mapper000;
mod mapper002;
mod mapper003;

use crate::{
    NesParseError,
    cartridge::{mapper000::Mapper000, mapper002::Mapper002, mapper003::Mapper003},
};
pub use cartridge_banks::{CartridgeBanks, *};
pub use cartridge_header::{CartridgeHeader, Mirroring};

/// Generic trait for underlying circuitry inside a catridge that will read and write to a catridge memory bank
pub trait Mapper {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8>;
    fn cpu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8);
    fn map_ppu(&self, address: u16) -> BankMapping;
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
    fn new(
        header: CartridgeHeader,
        mut banks: CartridgeBanks,
        trainer_data: Vec<u8>,
    ) -> Result<Self, NesParseError> {
        let mut mapper: Box<dyn Mapper> = match header.mapper_id {
            0 => Box::new(Mapper000::default()),
            2 => Box::new(Mapper002::default()),
            3 => Box::new(Mapper003::default()),
            id => return Err(NesParseError::UnsupportedMapper(id)),
        };

        for (i, byte) in trainer_data.iter().enumerate() {
            mapper.cpu_write(&mut banks, (0x7000 + i) as u16, *byte);
        }

        Ok(Cartridge {
            mapper,
            header,
            banks,
        })
    }

    pub fn from_nes(mut bytes: impl std::io::Read) -> Result<Self, NesParseError> {
        let mut header_data = [0; 16];
        bytes.read_exact(&mut header_data)?;
        let header = CartridgeHeader::from_nes(header_data)?;

        let mut trainer_data = vec![0; CartridgeHeader::TRAINER_SIZE];
        if header.has_trainer {
            bytes.read_exact(&mut trainer_data)?;
        }

        let mut prg_rom = vec![0; header.prg_rom_size];
        bytes.read_exact(&mut prg_rom)?;
        let mut chr_mem = vec![0; header.chr_mem_size];
        if header.chr_mem_is_rom {
            bytes.read_exact(&mut chr_mem)?;
        }

        let banks = CartridgeBanks::new(vec![0; header.prg_ram_size], prg_rom, chr_mem);
        Self::new(header, banks, trainer_data)
    }

    pub fn from_mapper(
        mapper_id: u16,
        prg_ram: Vec<u8>,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    ) -> Result<Self, NesParseError> {
        Self::new(
            CartridgeHeader {
                mapper_id,
                ..Default::default()
            },
            CartridgeBanks::new(prg_ram, prg_rom, chr_rom),
            vec![],
        )
    }

    pub fn cpu_read(&self, address: u16) -> Option<u8> {
        self.mapper.cpu_read(&self.banks, address)
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        self.mapper.cpu_write(&mut self.banks, address, value);
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        std::assert_matches!(address, 0x0000..=0x1fff);
        let mapping = self.mapper.map_ppu(address);
        self.banks.chr_mem.read(mapping, address)
    }

    pub fn ppu_write(&mut self, address: u16, value: u8) {
        std::assert_matches!(address, 0x0000..=0x1fff);
        let mapping = self.mapper.map_ppu(address);
        if !self.header.chr_mem_is_rom {
            self.banks.chr_mem.write(mapping, address, value);
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring().unwrap_or(self.header.mirroring)
    }

    pub fn header(&self) -> &CartridgeHeader {
        &self.header
    }
}
