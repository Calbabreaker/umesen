#[derive(Clone)]
pub struct Bus {
    pub ram: [u8; 0x10000],
    /// Cpu cycles counter for debugging
    pub cpu_cycles: u32,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            ram: [0; 0x10000],
            cpu_cycles: 0,
        }
    }
}

impl Bus {
    pub fn read_byte(&mut self, address: u16) -> u8 {
        self.clock_cpu();
        self.ram[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.clock_cpu();
        self.ram[address as usize] = value;
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let lsb = self.read_byte(address) as u16;
        let msb = self.read_byte(address + 1) as u16;
        msb << 8 | lsb
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let lsb = value as u8;
        let msb = (value << 8) as u8;
        self.write_byte(address, lsb);
        self.write_byte(address + 1, msb);
    }

    pub fn clock_cpu(&mut self) {
        // Every cpu clock is 12 master clocks
        self.cpu_cycles += 1;
    }
}
