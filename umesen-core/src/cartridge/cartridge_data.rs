use super::CartridgeHeader;

pub struct MemoryBank {
    memory: Vec<u8>,
}

impl MemoryBank {
    pub fn new(memory: Vec<u8>) -> Self {
        Self { memory }
    }

    pub fn mirrored_write(&mut self, address: u16, value: u8) {
        if self.memory.is_empty() {
            return;
        }
        let index = self.index(address);
        self.memory[index] = value;
    }

    pub fn mirrored_read(&self, address: u16) -> u8 {
        if self.memory.is_empty() {
            return 0;
        }
        let index = self.index(address);
        self.memory[index]
    }

    fn index(&self, address: u16) -> usize {
        (address as usize) % self.memory.len()
    }
}

pub struct CartridgeData {
    pub header: CartridgeHeader,
    pub prg_ram: MemoryBank,
    pub prg_rom: MemoryBank,
    /// chr_rom becomes 8 KiB of chr_ram if there is no chr_rom
    pub chr_mem: MemoryBank,
}

impl CartridgeData {
    pub fn new(header: CartridgeHeader, prg_rom: Vec<u8>, mut chr_rom: Vec<u8>) -> Self {
        if chr_rom.is_empty() {
            // Turn chr_rom into chr_ram
            chr_rom = vec![0; 8 * 1024];
        }

        Self {
            prg_ram: MemoryBank::new(vec![0; header.prg_ram_size]),
            header,
            prg_rom: MemoryBank::new(prg_rom),
            chr_mem: MemoryBank::new(chr_rom),
        }
    }
}
