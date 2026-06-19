use crate::cartridge::{Bank, BankMapping, CartridgeBanks, Mapper};

/// INES designation for CNROM boards
/// https://www.nesdev.org/wiki/CNROM
#[derive(Default, Debug)]
pub struct Mapper003 {
    bank_number: usize,
}

impl Mapper for Mapper003 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        match address {
            0x6000..=0x7fff => banks.prg_ram.read((8, Bank::Number(0)), address),
            0x8000..=0xffff => banks.prg_rom.read((32, Bank::Number(0)), address),
            _ => None,
        }
    }

    fn cpu_write(&mut self, _: &mut CartridgeBanks, address: u16, value: u8) {
        if let 0x8000..=0xffff = address {
            self.bank_number = value as usize;
        }
    }

    fn map_ppu(&self, _: u16) -> BankMapping {
        (8, Bank::Number(self.bank_number))
    }
}

#[cfg(test)]
mod test {
    use crate::Cartridge;

    #[test]
    fn test() {
        let mut prg_rom = vec![0; 32 * 1024];
        prg_rom[2] = 1;
        let mut chr_rom = vec![0; 32 * 1024];
        chr_rom[2] = 2;
        chr_rom[8 * 1024 * 2 + 2] = 1;
        let mut catridge = Cartridge::from_mapper(3, vec![], prg_rom, chr_rom).unwrap();

        assert_eq!(catridge.cpu_read(0x8002), Some(1));

        assert_eq!(catridge.ppu_read(0x0002), 2);

        catridge.cpu_write(0x8000, 2);
        assert_eq!(catridge.ppu_read(0x0002), 1);
    }
}
