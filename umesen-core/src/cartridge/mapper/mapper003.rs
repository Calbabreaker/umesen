use crate::cartridge::{Bank, BankMapping, Mapper};

/// INES designation for CNROM boards
/// https://www.nesdev.org/wiki/CNROM
#[derive(Default, Debug)]
pub struct Mapper003 {
    bank_number: u8,
}

impl Mapper for Mapper003 {
    fn map_cpu_read(&self, address: u16) -> Option<BankMapping> {
        Some(match address {
            0x8000..=0xffff => (32, Bank::Number(0)),
            _ => return None,
        })
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        if let 0x8000..=0xffff = address {
            self.bank_number = value;
        }
    }

    fn map_ppu(&self, _: u16) -> BankMapping {
        (8, Bank::Number(self.bank_number))
    }
}

#[cfg(test)]
mod test {
    use crate::cartridge::mapper::test::create_test_catridge;

    #[test]
    fn test() {
        let mut cartridge = create_test_catridge(3, 16, &[&[2]], 8, &[&[1], &[2]]);

        assert_eq!(cartridge.cpu_read(0x8000), Some(2));

        assert_eq!(cartridge.ppu_read(0x0000), 1);

        cartridge.cpu_write(0x8000, 1);
        assert_eq!(cartridge.ppu_read(0x0000), 2);
    }
}
