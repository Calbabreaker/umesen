use super::CartridgeHeader;

pub struct MemoryBank {
    pub data: Vec<u8>,
}

impl MemoryBank {
    pub fn new(memory: Vec<u8>) -> Self {
        Self { data: memory }
    }

    pub fn mirrored_write(&mut self, address: u16, value: u8) {
        if !self.data.is_empty() {
            let index = (address as usize) % self.data.len();
            self.data[index] = value;
        }
    }

    pub fn mirrored_read(&self, address: u16) -> u8 {
        if self.data.is_empty() {
            0
        } else {
            self.data[(address as usize) % self.data.len()]
        }
    }
}

pub struct CartridgeData {
    pub header: CartridgeHeader,
    pub prg_ram: MemoryBank,
    pub prg_rom: MemoryBank,
    pub chr_rom: MemoryBank,
    pub chr_ram: MemoryBank,
}

impl CartridgeData {
    pub fn new(header: CartridgeHeader, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        debug_assert_eq!(prg_rom.len(), header.prg_rom_size);
        debug_assert_eq!(chr_rom.len(), header.chr_rom_size);
        Self {
            prg_ram: MemoryBank::new(vec![0; header.prg_ram_size]),
            chr_ram: MemoryBank::new(vec![0; header.chr_ram_size]),
            prg_rom: MemoryBank::new(prg_rom),
            chr_rom: MemoryBank::new(chr_rom),
            header,
        }
    }
}
