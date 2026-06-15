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

/// Kilobytes
pub const KB: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub enum Bank {
    Number(usize),
    Last,
}

#[derive(Default)]
pub struct MemoryBanks(Vec<u8>);

impl MemoryBanks {
    pub fn write(&mut self, bank_size: usize, bank: Bank, offset: u16, value: u8) {
        if !self.0.is_empty() {
            let index = self.index(bank_size, bank, offset);
            self.0[index] = value;
        }
    }

    pub fn read(&self, bank_size: usize, bank: Bank, offset: u16) -> u8 {
        if !self.0.is_empty() {
            self.0[self.index(bank_size, bank, offset)]
        } else {
            0
        }
    }

    /// Get the index into the inner vec based on the banks and offset
    /// Offset will be wrapped around bank_size
    fn index(&self, bank_size: usize, bank: Bank, offset: u16) -> usize {
        assert!(bank_size.is_multiple_of(KB));
        let num_banks = self.0.len().div_ceil(bank_size);

        match bank {
            Bank::Number(number) => {
                bank_size * (number % num_banks) + (offset as usize % bank_size) % self.0.len()
            }
            Bank::Last => self.index(bank_size, Bank::Number(num_banks - 1), offset),
        }
    }
}

pub struct CartridgeBanks {
    pub prg_ram: MemoryBanks,
    pub prg_rom: MemoryBanks,
    pub chr_mem: MemoryBanks,
    chr_mem_is_rom: bool,
}

impl CartridgeBanks {
    pub fn new(prg_ram: Vec<u8>, prg_rom: Vec<u8>, chr_mem: Vec<u8>, chr_mem_is_rom: bool) -> Self {
        Self {
            prg_ram: MemoryBanks(prg_ram),
            prg_rom: MemoryBanks(prg_rom),
            chr_mem: MemoryBanks(chr_mem),
            chr_mem_is_rom,
        }
    }

    pub fn write_chr_mem(&mut self, bank_size: usize, bank: Bank, address: u16, value: u8) {
        if !self.chr_mem_is_rom {
            self.chr_mem.write(bank_size, bank, address, value)
        }
    }
}
