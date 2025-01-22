use crate::Cartridge;

pub struct PpuBus {
    palette_ram: [u8; 32],
    nametable_ram: [u8; 2048],
    pub cartridge: Option<Cartridge>,
}

impl Default for PpuBus {
    fn default() -> Self {
        Self {
            palette_ram: [0; 32],
            nametable_ram: [0; 2048],
            cartridge: None,
        }
    }
}

impl PpuBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1fff => match self.cartridge.as_ref() {
                Some(cartridge) => cartridge.ppu_read(address),
                None => 0,
            },
            0x2000..=0x3eff => 0,
            0x3f00..=0x3fff => 0,
            _ => 0,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => {
                if let Some(cartridge) = self.cartridge.as_ref() {
                    cartridge.ppu_write(address, value);
                }
            }
            0x2000..=0x3eff => (),
            0x3f00..=0x3fff => (),
            _ => (),
        }
    }
}
