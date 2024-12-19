pub struct Bus {
    ram: [u8; 0xffff],
}

impl Default for Bus {
    fn default() -> Self {
        Self { ram: [0; 0xffff] }
    }
}

impl Bus {
    pub fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let lsb = self.ram[address as usize] as u16;
        let msb = self.ram[address as usize + 1] as u16;
        msb << 8 | lsb
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let lsb = value as u8;
        let msb = (value << 8) as u8;
        self.ram[address as usize] = lsb;
        self.ram[address as usize + 1] = msb;
    }
}
