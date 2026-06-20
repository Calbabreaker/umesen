use crate::cartridge::{Bank, Mapper};

/// INES designation for UxROM boards
/// https://www.nesdev.org/wiki/UxROM
#[derive(Default, Debug)]
pub struct Mapper002 {
    bank_number_low: usize,
}

impl Mapper for Mapper002 {
    fn map_cpu_read(&self, address: u16) -> Option<crate::cartridge::BankMapping> {
        Some(match address {
            0x8000..=0xbfff => (16, Bank::Number(self.bank_number_low)),
            0xc000..=0xffff => (16, Bank::Last),
            _ => return None,
        })
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
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
    use crate::cartridge::mapper::test::create_test_catridge;

    #[test]
    fn test() {
        let mut cartridge = create_test_catridge(2, 16, &[&[1], &[2], &[3]], 8, &[&[2]]);
        assert_eq!(cartridge.ppu_read(0x0000), 2);

        assert_eq!(cartridge.cpu_read(0x8000), Some(1));
        assert_eq!(cartridge.cpu_read(0xc000), Some(3));

        cartridge.cpu_write(0x8000, 1);
        assert_eq!(cartridge.cpu_read(0x8000), Some(2));
        assert_eq!(cartridge.cpu_read(0xc000), Some(3));
    }
}
