use crate::cartridge::{Bank, BankMapping, Mapper};

/// INES designation for NROM boards
/// https://www.nesdev.org/wiki/NROM
#[derive(Default, Debug)]
pub struct Mapper000 {}

impl Mapper for Mapper000 {
    fn map_cpu_read(&self, address: u16) -> Option<BankMapping> {
        match address {
            0x8000..=0xffff => Some((32, Bank::Number(0))),
            _ => None,
        }
    }

    fn cpu_write(&mut self, _: u16, _: u8) {}

    fn map_ppu(&self, _: u16) -> BankMapping {
        (8, Bank::Number(0))
    }
}

#[cfg(test)]
mod test {
    use crate::cartridge::mapper::test::create_test_catridge;

    #[test]
    fn prg_ram() {
        let mut cartridge = create_test_catridge(0, 16, &[], 8, &[]);
        cartridge.cpu_write(0x6000, 2);
        assert_eq!(cartridge.cpu_read(0x6000), Some(2));
    }

    #[test]
    fn test() {
        let cartridge = create_test_catridge(0, 16, &[&[1, 2, 3]], 8, &[&[69]]);
        assert_eq!(cartridge.cpu_read(0x8000), Some(1));
        assert_eq!(cartridge.cpu_read(0x8002), Some(3));
        assert_eq!(cartridge.cpu_read(0xc002), Some(3));

        assert_eq!(cartridge.ppu_read(0x0000), 69);
    }
}
