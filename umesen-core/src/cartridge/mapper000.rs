use crate::cartridge::{CartridgeBanks, Mapper, MemoryBankExt};

/// INES designation for NROM boards
/// https://www.nesdev.org/wiki/NROM
#[derive(Default)]
pub struct Mapper000 {}

impl Mapper for Mapper000 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> u8 {
        match address {
            0x6000..=0x7fff => banks.prg_ram.mirrored_read(address - 0x6000),
            0x8000..=0xffff => banks.prg_rom.mirrored_read(address - 0x8000),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        if let 0x6000..=0x7fff = address {
            banks.prg_ram.mirrored_write(address - 0x6000, value)
        }
    }

    fn ppu_read(&self, banks: &CartridgeBanks, address: u16) -> u8 {
        if banks.chr_rom.is_empty() {
            banks.chr_ram.mirrored_read(address)
        } else {
            banks.chr_rom.mirrored_read(address)
        }
    }

    fn ppu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        if banks.chr_rom.is_empty() {
            banks.chr_ram.mirrored_write(address, value);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Cartridge;

    #[test]
    fn test() {
        let mut catridge = Cartridge::with_rom(0, vec![1, 2, 3], vec![1, 2, 3], 69);
        assert_eq!(catridge.cpu_read(0x8000), 1);
        assert_eq!(catridge.cpu_read(0x8003), 1);
        catridge.cpu_write(0x6000, 2);
        assert_eq!(catridge.cpu_read(0x6000), 2);
        assert_eq!(catridge.ppu_read(0x0000), 1);
    }
}
