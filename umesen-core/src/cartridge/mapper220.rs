use crate::cartridge::{CartridgeBanks, Mapper, MemoryBankExt};

/// Mapper is not assigned by INES to anything useful so this will be used as a mapper for testing
/// This is just going to have ram
#[derive(Default)]
pub struct Mapper220 {}

#[allow(unused)]
impl Mapper for Mapper220 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        Some(banks.prg_ram.mirrored_read(address))
    }

    fn cpu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        banks.prg_ram.mirrored_write(address, value)
    }

    fn ppu_read(&self, banks: &CartridgeBanks, address: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {}
}
