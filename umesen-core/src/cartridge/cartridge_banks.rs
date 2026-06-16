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

#[derive(Debug, Clone, Copy)]
pub enum Bank {
    Number(usize),
    Last,
}

#[derive(Default)]
pub struct MemoryBanks(Vec<u8>);

// (size of a single bank in units of kb, and the bank number)
pub type BankMapping = (usize, Bank);

impl MemoryBanks {
    pub fn write(&mut self, bank_mapping: BankMapping, offset: u16, value: u8) {
        if !self.0.is_empty() {
            let index = self.index(bank_mapping, offset);
            self.0[index] = value;
        }
    }

    pub fn read(&self, bank_mapping: BankMapping, offset: u16) -> u8 {
        if !self.0.is_empty() {
            self.0[self.index(bank_mapping, offset)]
        } else {
            0
        }
    }

    /// Get the index into the inner vec based on the banks and offset
    /// Offset will be wrapped around bank_size
    fn index(&self, (bank_size_kb, bank): BankMapping, offset: u16) -> usize {
        let bank_size = bank_size_kb * 1024;
        let num_banks = self.0.len().div_ceil(bank_size);

        match bank {
            Bank::Number(number) => {
                (bank_size * (number % num_banks) + (offset as usize % bank_size)) % self.0.len()
            }
            Bank::Last => self.index((bank_size_kb, Bank::Number(num_banks - 1)), offset),
        }
    }
}

pub struct CartridgeBanks {
    pub prg_ram: MemoryBanks,
    pub prg_rom: MemoryBanks,
    pub chr_mem: MemoryBanks,
}

impl CartridgeBanks {
    pub fn new(prg_ram: Vec<u8>, prg_rom: Vec<u8>, chr_mem: Vec<u8>) -> Self {
        Self {
            prg_ram: MemoryBanks(prg_ram),
            prg_rom: MemoryBanks(prg_rom),
            chr_mem: MemoryBanks(chr_mem),
        }
    }
}
