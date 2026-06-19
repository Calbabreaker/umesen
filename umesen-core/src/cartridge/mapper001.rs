use crate::cartridge::{Bank, BankMapping, CartridgeBanks, Mapper, Mirroring};

// 0b0010_0000
const SHIFT_CHECK_BIT_POS: u8 = 5;

/// INES designation for MMC1 boards
/// https://www.nesdev.org/wiki/MMC1
#[derive(Default, Debug)]
pub struct Mapper001 {
    shift_register: u8,
    control_register: u8,
    prg_bank_number: u8,
    chr_bank_number_0: u8,
    chr_bank_number_1: u8,
}

impl Mapper001 {
    fn write_load_register(&mut self, address: u16, value: u8) {
        if value & 0b1000_0000 != 0 {
            self.reset();
        } else {
            self.shift_register >>= 1;
            self.shift_register |= (value & 0b1) << SHIFT_CHECK_BIT_POS;
            if self.shift_register & 0b1 != 0 {
                self.shift_register >>= 1;
                match address {
                    0x8000..=0x9fff => self.control_register = self.shift_register,
                    0xa000..=0xbfff => self.chr_bank_number_0 = self.shift_register,
                    0xc000..=0xdfff => self.chr_bank_number_1 = self.shift_register,
                    0xe000..=0xffff => self.prg_bank_number = self.shift_register,
                    _ => unreachable!(),
                }
                self.shift_register = 1 << SHIFT_CHECK_BIT_POS;
            }
        }
    }
}

impl Mapper for Mapper001 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        let bank_number = self.prg_bank_number as usize;
        let (bank_8000, bank_c000) = match self.control_register & 0b01100 {
            // 32 kib mode
            0b00000 | 0b00100 => split_large_bank(bank_number),
            // 16 kib, 8000 = last bank
            0b01000 => ((Bank::Last), (Bank::Number(bank_number))),
            // 16 kib, c000 = last bank
            0b01100 => (Bank::Number(bank_number), (Bank::Last)),
            _ => unreachable!(),
        };

        match address {
            0x6000..=0x7fff => banks.prg_ram.read((8, Bank::Number(0)), address),
            0x8000..=0xbfff => banks.prg_rom.read((16, bank_8000), address),
            0xc000..=0xffff => banks.prg_rom.read((16, bank_c000), address),
            _ => None,
        }
    }

    fn cpu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        match address {
            0x6000..=0x7fff => banks.prg_ram.write((8, Bank::Number(0)), address, value),
            0x8000..=0xffff => self.write_load_register(address, value),
            _ => (),
        }
    }

    fn map_ppu(&self, address: u16) -> BankMapping {
        let bank_number_0 = self.chr_bank_number_0 as usize;
        let bank_number_1 = self.chr_bank_number_1 as usize;
        let (bank_0000, bank_1000) = match self.control_register & 0b10000 {
            // 8 Kib mode
            0b00000 => split_large_bank(bank_number_0),
            // 4 Kib split mode
            0b10000 => (Bank::Number(bank_number_0), Bank::Number(bank_number_1)),
            _ => unreachable!(),
        };

        match address {
            0x0000..=0x0fff => (4, bank_0000),
            0x1000..=0x1fff => (4, bank_1000),
            _ => unreachable!(),
        }
    }

    fn reset(&mut self) {
        // Keep a one there so we can just check bit 5 to see if register written 5 times
        self.shift_register = 1 << SHIFT_CHECK_BIT_POS;
        // Switch to bank mode 3
        self.control_register |= 0b01100;
    }

    fn mirroring(&self) -> Option<Mirroring> {
        Some(match self.control_register & 0b11 {
            0 => Mirroring::SingleScreenLow,
            1 => Mirroring::SingleScreenHigh,
            2 => Mirroring::Horizontal,
            3 => Mirroring::Vertical,
            _ => unreachable!(),
        })
    }
}

fn split_large_bank(bank_number: usize) -> (Bank, Bank) {
    (
        Bank::Number(bank_number & 0b1110),
        Bank::Number((bank_number & 0b1110) + 1),
    )
}

#[cfg(test)]
mod test {
    use crate::{Cartridge, cartridge::Mirroring};

    fn write_register(catridge: &mut Cartridge, address: u16, bits: u8) {
        for i in 0..5 {
            catridge.cpu_write(address, bits >> i);
        }
    }

    fn setup_catridge() -> Cartridge {
        let mut prg_rom = vec![0; 128 * 1024];
        prg_rom[2] = 2;
        prg_rom[16 * 1024 + 2] = 9; // 2nd bank
        prg_rom[16 * 1024 * 2 + 2] = 1; // 3rd bank
        *prg_rom.last_mut().unwrap() = 3;
        let mut chr_rom = vec![0; 32 * 1024];
        chr_rom[2] = 2;
        chr_rom[4 * 1024 + 2] = 9; // 2nd bank
        chr_rom[4 * 1024 * 2 + 2] = 1; // 3rd bank
        Cartridge::from_mapper(1, vec![0; 1024], prg_rom, chr_rom).unwrap()
    }

    #[test]
    fn shift_and_control_register() {
        let mut catridge = setup_catridge();
        assert_eq!(catridge.cpu_read(0xffff), Some(3));

        // Write control register
        catridge.cpu_write(0x9999, 0b0001_1001);
        catridge.cpu_write(0x8000, 0b0001_0010);
        catridge.cpu_write(0x8000, 0b0000_0000);
        catridge.cpu_write(0xcccc, 0b0000_0000);
        catridge.cpu_write(0x8000, 0b0000_0000);
        assert_eq!(catridge.mirroring(), Mirroring::SingleScreenHigh);
        assert_eq!(catridge.cpu_read(0xffff), Some(0));

        catridge.cpu_write(0x8000, 0b0000_0001);
        catridge.cpu_write(0x8000, 0b0000_0001);
        catridge.cpu_write(0x8000, 0b1000_0000); // Reset
        assert_eq!(catridge.cpu_read(0xffff), Some(3));
        catridge.cpu_write(0xcccc, 0b0000_0000);
        catridge.cpu_write(0x8000, 0b0000_0000);
        catridge.cpu_write(0x8000, 0b0000_0000);
        assert_eq!(catridge.mirroring(), Mirroring::SingleScreenHigh);
    }

    #[test]
    fn prg_banks() {
        let mut catridge = setup_catridge();
        // 0xc000 last bank
        write_register(&mut catridge, 0x8000, 0b11111);
        assert_eq!(catridge.cpu_read(0xffff), Some(3));
        assert_eq!(catridge.cpu_read(0x8002), Some(2));
        write_register(&mut catridge, 0xe000, 0b00010);
        assert_eq!(catridge.cpu_read(0x8002), Some(1));

        // 0x8000 last bank
        write_register(&mut catridge, 0x8000, 0b01000);
        assert_eq!(catridge.cpu_read(0xbfff), Some(3));
        assert_eq!(catridge.cpu_read(0xc002), Some(1));

        // 32 KB mode
        write_register(&mut catridge, 0x8000, 0b00100);
        assert_eq!(catridge.cpu_read(0xffff), Some(0));
        assert_eq!(catridge.cpu_read(0xbfff), Some(0));
        assert_eq!(catridge.cpu_read(0x8002), Some(1));
        write_register(&mut catridge, 0xe000, 0b00001);
        assert_eq!(catridge.cpu_read(0x8002), Some(2));
        assert_eq!(catridge.cpu_read(0xc002), Some(9));

        // prg ram
        catridge.cpu_write(0x6969, 69);
        assert_eq!(catridge.cpu_read(0x6969), Some(69));
    }

    #[test]
    fn chr_banks() {
        let mut catridge = setup_catridge();
        write_register(&mut catridge, 0x8000, 0b11111);
        write_register(&mut catridge, 0xa000, 0b00001);
        write_register(&mut catridge, 0xc000, 0b00010);
        assert_eq!(catridge.ppu_read(0x0002), 9);
        assert_eq!(catridge.ppu_read(0x1002), 1);

        write_register(&mut catridge, 0x8000, 0b00000);
        write_register(&mut catridge, 0xa000, 0b00001);
        assert_eq!(catridge.ppu_read(0x0002), 2);
        assert_eq!(catridge.ppu_read(0x1002), 9);
    }
}
