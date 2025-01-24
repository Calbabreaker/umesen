use super::CartridgeHeader;

/// Wrapper around a normal slice but allows for deriving Default for an arbitrary size at compile time
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

pub trait MemoryBankExt {
    fn mirrored_write(&mut self, address: u16, value: u8);
    fn mirrored_read(&self, address: u16) -> u8;
}

macro_rules! impl_memory_bank_ext_for {
    ($($type:ty),+) => {
        $(impl MemoryBankExt for $type {
            fn mirrored_write(&mut self, address: u16, value: u8) {
                if !self.is_empty() {
                    let index = (address as usize) % self.len();
                    self[index] = value;
                }
            }

            fn mirrored_read(&self, address: u16) -> u8 {
                if self.is_empty() {
                    0
                } else {
                    self[(address as usize) % self.len()]
                }
            }
        })*
    };
}

impl_memory_bank_ext_for!(Vec<u8>, [u8]);

pub struct CartridgeBanks {
    pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_ram: Vec<u8>,
}

impl CartridgeBanks {
    pub fn new(header: &CartridgeHeader, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        debug_assert_eq!(prg_rom.len(), header.prg_rom_size);
        debug_assert_eq!(chr_rom.len(), header.chr_rom_size);
        Self {
            prg_ram: vec![0; header.prg_ram_size],
            chr_ram: vec![0; header.chr_ram_size],
            prg_rom,
            chr_rom,
        }
    }
}
