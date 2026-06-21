use crate::cartridge::{Bank, BankMapping, Mapper, Mirroring};

/// INES designation for MMC3 boards
/// https://www.nesdev.org/wiki/MMC3
#[derive(Default, Debug)]
pub struct Mapper004 {
    mirroring: Mirroring,
    registers: [u8; 8],
    selected_register_index: usize,
    prg_mode_flip: bool,
    chr_mode_flip: bool,

    irq_counter: u8,
    irq_reload: bool,
    irq_latch_value: u8,
    irq_status: bool,
    irq_enable: bool,
}

impl Mapper for Mapper004 {
    fn map_cpu_read(&self, address: u16) -> Option<BankMapping> {
        let bank_order = [
            Bank::Number(self.registers[6]),
            Bank::Number(self.registers[7]),
            Bank::FromLast(1),
        ];
        match address {
            0x8000..=0xdfff => {
                let mut i = (address as usize >> 13) & 0b11;
                if self.prg_mode_flip {
                    i = bank_order.len() - i - 1;
                }
                Some((8, bank_order[i]))
            }
            0xe000..=0xffff => Some((8, Bank::FromLast(0))),
            _ => None,
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        let even = address.is_multiple_of(2);
        match address {
            // Bank select
            0x8000..=0x9fff if even => {
                self.selected_register_index = (value & 0b0000_0111) as usize;
                self.prg_mode_flip = value & 0b0100_0000 != 0;
                self.chr_mode_flip = value & 0b1000_0000 != 0;
            }
            // Bank register data
            0x8000..=0x9fff if !even => {
                self.registers[self.selected_register_index] = value;
            }
            // Mirroring
            0xa000..=0xbfff if even => {
                self.mirroring = if value & 0b1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                }
            }
            0xc000..=0xdfff if even => self.irq_latch_value = value,
            0xc000..=0xdfff if !even => self.irq_reload = true,
            // Disable IRQs and acknowledge them
            0xe000..=0xffff if even => {
                self.irq_enable = false;
                self.irq_status = false;
            }
            0xe000..=0xffff if !even => self.irq_enable = true,
            _ => (),
        }
    }

    fn map_ppu(&self, address: u16) -> BankMapping {
        let mut section_0 = [
            Bank::Number(self.registers[0] & !1),
            Bank::Number(self.registers[0] | 1),
            Bank::Number(self.registers[1] & !1),
            Bank::Number(self.registers[1] | 1),
        ];
        let mut section_1 = [
            Bank::Number(self.registers[2]),
            Bank::Number(self.registers[3]),
            Bank::Number(self.registers[4]),
            Bank::Number(self.registers[5]),
        ];
        if self.chr_mode_flip {
            std::mem::swap(&mut section_0, &mut section_1);
        }
        let i = address as usize / 0x400;
        match address {
            0x0000..=0x0fff => (1, section_0[i]),
            0x1000..=0x1fff => (1, section_1[i - 4]),
            _ => unreachable!(),
        }
    }

    fn irq_status(&self) -> bool {
        self.irq_status
    }

    fn signal_scanline(&mut self) {
        if self.irq_reload {
            self.irq_counter = self.irq_latch_value;
            self.irq_reload = false;
            return;
        }

        if self.irq_counter == 0 {
            if self.irq_enable {
                self.irq_status = true;
            }
            self.irq_counter = self.irq_latch_value;
        } else {
            self.irq_counter -= 1;
        }
    }

    fn mirroring(&self) -> Option<Mirroring> {
        Some(self.mirroring)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        Cartridge,
        cartridge::{Mirroring, mapper::test::create_test_catridge},
    };

    fn setup_catridge() -> Cartridge {
        create_test_catridge(4, 8, &[&[1, 69], &[2], &[3], &[4]], 4, &[&[6], &[7], &[8]])
    }

    #[test]
    fn mirroring() {
        let mut cartridge = setup_catridge();
        cartridge.cpu_write(0xa000, 0b100);
        assert_eq!(cartridge.mirroring(), Mirroring::Vertical);
        cartridge.cpu_write(0xa000, 0b101);
        assert_eq!(cartridge.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn prg_rom() {
        let mut cartridge = setup_catridge();
        assert_eq!(cartridge.cpu_read(0xe000), Some(4));

        // 0xc000 2nd last bank
        cartridge.cpu_write(0x8000, 0b0000_0110);
        cartridge.cpu_write(0x8001, 1);
        cartridge.cpu_write(0x8000, 0b0000_0111);
        cartridge.cpu_write(0x8001, 0);
        assert_eq!(cartridge.cpu_read(0x8000), Some(2));
        assert_eq!(cartridge.cpu_read(0xa001), Some(69));
        assert_eq!(cartridge.cpu_read(0xc000), Some(3));
        assert_eq!(cartridge.cpu_read(0xe000), Some(4));

        cartridge.cpu_write(0x8000, 0b0100_0000);
        assert_eq!(cartridge.cpu_read(0x8000), Some(3));
        assert_eq!(cartridge.cpu_read(0xa001), Some(69));
        assert_eq!(cartridge.cpu_read(0xc000), Some(2));
        assert_eq!(cartridge.cpu_read(0xe000), Some(4));
    }

    #[test]
    fn irq() {
        let mut cartridge = setup_catridge();
        cartridge.cpu_write(0xc000, 1);
        cartridge.cpu_write(0xc001, 0);
        cartridge.cpu_write(0xe001, 0);

        cartridge.signal_scanline();
        assert!(!cartridge.irq_status());
        cartridge.signal_scanline();
        assert!(!cartridge.irq_status());
        cartridge.signal_scanline();
        assert!(cartridge.irq_status());

        cartridge.cpu_write(0xe001, 0);
        cartridge.signal_scanline();
        cartridge.signal_scanline();
        assert!(cartridge.irq_status());
    }
}
