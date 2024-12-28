use crate::NesParseError;

use super::CartridgeHeader;

#[derive(Default)]
pub struct CartridgeData {
    pub header: CartridgeHeader,
    pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    /// chr_rom becomes 8 KiB of chr_ram if there is no chr_rom
    pub chr_mem: Vec<u8>,
}

impl CartridgeData {
    pub fn from_nes(
        header: CartridgeHeader,
        mut data: impl std::io::Read,
    ) -> Result<Self, NesParseError> {
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

        Ok(Self::new(header, prg_rom, chr_rom))
    }

    pub fn new(header: CartridgeHeader, prg_rom: Vec<u8>, mut chr_rom: Vec<u8>) -> Self {
        if chr_rom.is_empty() {
            // Turn chr_rom into chr_ram
            chr_rom = vec![0; 8 * 1024];
        }

        Self {
            prg_ram: vec![0; header.prg_ram_size],
            header,
            prg_rom,
            chr_mem: chr_rom,
        }
    }
}


