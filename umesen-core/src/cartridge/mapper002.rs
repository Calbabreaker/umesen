use crate::cartridge::{Bank, CartridgeBanks, KB, Mapper};

/// INES designation for UxROM boards
/// https://www.nesdev.org/wiki/UxROM
#[derive(Default)]
pub struct Mapper002 {
    bank_number_low: usize,
}

impl Mapper for Mapper002 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        Some(match address {
            0x8000..=0xbfff => {
                banks
                    .prg_rom
                    .read(16 * KB, Bank::Number(self.bank_number_low), address)
            }
            0xc000..=0xffff => banks.prg_rom.read(16 * KB, Bank::Last, address),
            _ => return None,
        })
    }

    fn cpu_write(&mut self, _: &mut CartridgeBanks, address: u16, value: u8) {
        if let 0x8000..=0xffff = address {
            self.bank_number_low = value as usize;
        }
    }

    fn ppu_read(&self, banks: &CartridgeBanks, address: u16) -> u8 {
        banks.chr_mem.read(8 * KB, Bank::Number(0), address)
    }

    fn ppu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        banks.write_chr_mem(8 * KB, Bank::Number(0), address, value);
    }
}

#[cfg(test)]
mod test {
    use crate::{Cartridge, cartridge::KB};

    #[test]
    fn test() {
        let mut prg_rom = vec![0; 128 * KB];
        prg_rom[2] = 2;
        prg_rom[16 * KB * 2 + 2] = 1;
        *prg_rom.last_mut().unwrap() = 3;
        let mut chr_rom = vec![0; 4 * KB];
        chr_rom[2] = 1;
        let mut catridge = Cartridge::from_mapper(2, vec![0; 1024], prg_rom, chr_rom).unwrap();

        assert_eq!(catridge.ppu_read(0x0002), 1);

        assert_eq!(catridge.cpu_read(0x8002), Some(2));
        assert_eq!(catridge.cpu_read(0xffff), Some(3));

        catridge.cpu_write(0x8000, 2);
        assert_eq!(catridge.cpu_read(0x8002), Some(1));
        assert_eq!(catridge.cpu_read(0xffff), Some(3));
    }
}
