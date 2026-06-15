use crate::cartridge::{Bank, CartridgeBanks, KB, Mapper};

/// INES designation for CNROM boards
/// https://www.nesdev.org/wiki/CNROM
#[derive(Default)]
pub struct Mapper003 {
    bank_number: usize,
}

impl Mapper for Mapper003 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        Some(match address {
            0x6000..=0x7fff => banks.prg_ram.read(2 * KB, Bank::Number(0), address),
            0x8000..=0xffff => banks.prg_rom.read(32 * KB, Bank::Number(0), address),
            _ => return None,
        })
    }

    fn cpu_write(&mut self, _: &mut CartridgeBanks, address: u16, value: u8) {
        if let 0x8000..=0xffff = address {
            self.bank_number = value as usize;
        }
    }

    fn ppu_read(&self, banks: &CartridgeBanks, address: u16) -> u8 {
        banks
            .chr_mem
            .read(8 * KB, Bank::Number(self.bank_number), address)
    }

    fn ppu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        banks.write_chr_mem(8 * KB, Bank::Number(self.bank_number), address, value);
    }
}

#[cfg(test)]
mod test {
    use crate::{Cartridge, cartridge::KB};

    #[test]
    fn test() {
        let mut prg_rom = vec![0; 32 * KB];
        prg_rom[2] = 1;
        let mut chr_rom = vec![0; 32 * KB];
        chr_rom[2] = 2;
        chr_rom[8 * KB * 2 + 2] = 1;
        let mut catridge = Cartridge::from_mapper(3, vec![0; KB], prg_rom, chr_rom).unwrap();

        assert_eq!(catridge.cpu_read(0x8002), Some(1));

        assert_eq!(catridge.ppu_read(0x0002), 2);

        catridge.cpu_write(0x8000, 2);
        assert_eq!(catridge.ppu_read(0x0002), 1);
    }
}
