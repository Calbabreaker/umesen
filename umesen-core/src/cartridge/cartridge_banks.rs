/// Wrapper around a normal slice but allows for deriving Default for an arbitrary size at compile time
/// because rust devs are too pedantic https://github.com/rust-lang/rust/issues/61415
#[derive(Copy, Clone, Debug)]
pub struct FixedArray<T, const C: usize>([T; C]);

impl<T: Default + Copy, const C: usize> Default for FixedArray<T, C> {
    fn default() -> Self {
        Self([Default::default(); C])
    }
}

impl<T, const C: usize> std::ops::Deref for FixedArray<T, C> {
    type Target = [T; C];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const C: usize> std::ops::DerefMut for FixedArray<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
pub struct MemoryBank(Vec<u8>);

impl MemoryBank {
    pub fn mirrored_write(&mut self, address: u16, value: u8) {
        if !self.0.is_empty() {
            let index = (address as usize) % self.0.len();
            self.0[index] = value;
        }
    }

    pub fn mirrored_read(&self, address: u16) -> u8 {
        if self.0.is_empty() {
            0
        } else {
            self.0[(address as usize) % self.0.len()]
        }
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}

pub struct CartridgeBanks {
    pub prg_ram: MemoryBank,
    pub prg_rom: MemoryBank,
    pub chr_rom: MemoryBank,
    pub chr_ram: MemoryBank,
}

impl CartridgeBanks {
    pub fn new(
        prg_ram_size: usize,
        chr_ram_size: usize,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    ) -> Self {
        Self {
            prg_ram: MemoryBank(vec![0; prg_ram_size]),
            chr_ram: MemoryBank(vec![0; chr_ram_size]),
            prg_rom: MemoryBank(prg_rom),
            chr_rom: MemoryBank(chr_rom),
        }
    }
}
