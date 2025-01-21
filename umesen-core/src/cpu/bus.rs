use crate::{cartridge::Catridge, Ppu};

pub struct CpuBus {
    // 2kb of cpu ram
    pub ram: [u8; 2048],
    /// Cpu cycles counter for debugging
    pub cpu_cycles: u32,
    pub ppu: Ppu,
    pub cartridge: Option<Catridge>,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 2048],
            cpu_cycles: 0,
            cartridge: None,
            ppu: Ppu::default(),
        }
    }
}

impl std::fmt::Debug for CpuBus {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl CpuBus {
    /// https://www.nesdev.org/wiki/CPU_memory_map
    pub fn unclocked_read_byte(&self, address: u16) -> u8 {
        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram[(address as usize) % self.ram.len()],
            0x2000..=0x3fff => self.ppu.registers.read(address),
            0x4020..=0xffff => {
                if let Some(catridge) = self.cartridge.as_ref() {
                    catridge.cpu_read(address)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
    pub fn read_byte(&mut self, address: u16) -> u8 {
        let byte = self.unclocked_read_byte(address);
        self.clock();
        byte
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // 2kb of ram is mirrored 3 times
            0x0000..=0x1fff => self.ram[(address as usize) % self.ram.len()] = value,
            0x2000..=0x3fff => self.ppu.registers.write(address, value),
            0x4020..=0xffff => {
                if let Some(cartridge) = self.cartridge.as_mut() {
                    cartridge.cpu_write(address, value)
                };
            }
            _ => (),
        }
        self.clock();
    }

    pub fn unclocked_read_word(&self, address: u16) -> u16 {
        let lsb = self.unclocked_read_byte(address) as u16;
        let msb = self.unclocked_read_byte(address + 1) as u16;
        (msb << 8) | lsb
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let lsb = self.read_byte(address) as u16;
        let msb = self.read_byte(address + 1) as u16;
        (msb << 8) | lsb
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let lsb = value as u8;
        let msb = (value << 8) as u8;
        self.write_byte(address, lsb);
        self.write_byte(address + 1, msb);
    }

    pub fn clock(&mut self) {
        // Every cpu clock is 12 master clocks
        self.cpu_cycles += 1;
        for _ in 0..3 {}
    }

    pub fn attach_catridge(&mut self, catridge: Catridge) {
        self.cartridge = Some(catridge);
    }
}
