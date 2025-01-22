use crate::cartridge::{CartridgeData, Mapper, MemoryBankExt};

/// INES designation for NROM boards
/// https://www.nesdev.org/wiki/NROM
pub struct Mapper000 {
    data: CartridgeData,
}

impl Mapper000 {
    pub fn new(data: CartridgeData) -> Self {
        Self { data }
    }
}

impl Mapper for Mapper000 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7fff => self.data.prg_ram.mirrored_read(address - 0x6000),
            0x8000..=0xffff => self.data.prg_rom.mirrored_read(address - 0x8000),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        if let 0x6000..=0x7fff = address {
            self.data.prg_ram.mirrored_write(address - 0x6000, value)
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        if self.data.chr_rom.is_empty() {
            self.data.chr_ram.mirrored_read(address)
        } else {
            self.data.chr_rom.mirrored_read(address)
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        if self.data.chr_rom.is_empty() {
            self.data.chr_ram.mirrored_write(address, value);
        } else {
            self.data.chr_rom.mirrored_write(address, value);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Cartridge;

    #[test]
    fn test() {
        let catridge = Cartridge::new(0, vec![1, 2, 3], vec![1, 2, 3], 69);
        assert_eq!(catridge.cpu_read(0x8000), 1);
        assert_eq!(catridge.cpu_read(0x8003), 1);
        catridge.cpu_write(0x6000, 2);
        assert_eq!(catridge.cpu_read(0x6000), 2);
        assert_eq!(catridge.ppu_read(0x0000), 1);
    }
}
