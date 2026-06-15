use crate::cartridge::{Bank, CartridgeBanks, Mapper, SIZE_8KB, SIZE_32KB};

/// INES designation for NROM boards
/// https://www.nesdev.org/wiki/NROM
#[derive(Default)]
pub struct Mapper000 {}

impl Mapper for Mapper000 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        Some(match address {
            0x6000..=0x7fff => banks.prg_ram.read(SIZE_8KB, Bank::Number(0), address),
            0x8000..=0xffff => banks.prg_rom.read(SIZE_32KB, Bank::Number(0), address),
            _ => return None,
        })
    }

    fn cpu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        if let 0x6000..=0x7fff = address {
            banks
                .prg_ram
                .write(SIZE_8KB, Bank::Number(0), address, value)
        }
    }

    fn ppu_read(&self, banks: &CartridgeBanks, address: u16) -> u8 {
        banks.chr_mem.read(SIZE_8KB, Bank::Number(0), address)
    }

    fn ppu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        banks.write_chr_mem(SIZE_8KB, Bank::Number(0), address, value);
    }
}

#[cfg(test)]
mod test {
    use crate::Cartridge;

    #[test]
    fn test() {
        let mut prg_rom = vec![0; 16 * 1024];
        prg_rom[2] = 2;
        let mut chr_rom = vec![0; 4 * 1024];
        chr_rom[2] = 1;
        let mut catridge = Cartridge::from_mapper(0, vec![0; 1024], prg_rom, chr_rom).unwrap();
        catridge.cpu_write(0x6000, 2);
        assert_eq!(catridge.cpu_read(0x6000), Some(2));

        assert_eq!(catridge.cpu_read(0x8002), Some(2));
        assert_eq!(catridge.cpu_read(0xc002), Some(2));

        assert_eq!(catridge.ppu_read(0x0002), 1);
    }
}
