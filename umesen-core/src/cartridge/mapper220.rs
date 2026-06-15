use crate::cartridge::{Bank, CartridgeBanks, Mapper, SIZE_16KB};

/// Mapper is not assigned by INES to anything useful so this will be used as a mapper for testing
/// This is just going to have ram
#[derive(Default)]
pub struct Mapper220 {}

#[allow(unused)]
impl Mapper for Mapper220 {
    fn cpu_read(&self, banks: &CartridgeBanks, address: u16) -> Option<u8> {
        Some(banks.prg_ram.read(SIZE_16KB, Bank::Number(0), address))
    }

    fn cpu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {
        banks
            .prg_ram
            .write(SIZE_16KB, Bank::Number(0), address, value)
    }

    fn ppu_read(&self, banks: &CartridgeBanks, address: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, banks: &mut CartridgeBanks, address: u16, value: u8) {}
}
