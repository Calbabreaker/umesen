use crate::cartridge::{Bank, CartridgeBanks, Mapper};

/// INES designation for UxROM boards
/// https://www.nesdev.org/wiki/UxROM
#[derive(Default)]
pub struct Mapper002 {
    bank_number_low: usize,
}

impl Mapper for Mapper002 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        match address {
            0x8000..=0xbfff => banks
                .prg_rom
                .read((16, Bank::Number(self.bank_number_low)), address),
            0xc000..=0xffff => banks.prg_rom.read((16, Bank::Last), address),
            _ => None,
        }
    }

    fn cpu_write(&mut self, _: &mut CartridgeBanks, address: u16, value: u8) {
        if let 0x8000..=0xffff = address {
            self.bank_number_low = value as usize;
        }
    }

    fn map_ppu(&self, _: u16) -> super::BankMapping {
        (8, Bank::Number(0))
    }
}

#[cfg(test)]
mod test {
    use crate::Cartridge;

    #[test]
    fn test() {
        let mut prg_rom = vec![0; 128 * 1024];
        prg_rom[2] = 2;
        prg_rom[16 * 1024 * 2 + 2] = 1; // 3rd bank
        *prg_rom.last_mut().unwrap() = 3;
        let mut chr_rom = vec![0; 4 * 1024];
        chr_rom[2] = 1;
        let mut catridge = Cartridge::from_mapper(2, vec![], prg_rom, chr_rom).unwrap();

        assert_eq!(catridge.ppu_read(0x0002), 1);

        assert_eq!(catridge.cpu_read(0x8002), Some(2));
        assert_eq!(catridge.cpu_read(0xffff), Some(3));

        catridge.cpu_write(0x8000, 2);
        assert_eq!(catridge.cpu_read(0x8002), Some(1));
        assert_eq!(catridge.cpu_read(0xffff), Some(3));
    }
}
