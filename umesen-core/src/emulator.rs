use crate::{cartridge::FixedArray, ppu, Cartridge, Cpu, NesParseError};

/// High level struct for controlling the cpu
#[derive(Default)]
pub struct Emulator {
    pub cpu: Cpu,
}

impl Emulator {
    pub fn next_frame(&mut self) -> &FixedArray<u32, { ppu::WIDTH * ppu::HEIGHT }> {
        while !self.cpu.bus.ppu.frame_complete {
            if let Err(err) = self.cpu.execute_next() {
                log::error!("{err}")
            }
        }
        self.cpu.bus.ppu.frame_complete = false;
        &self.cpu.bus.ppu.screen_pixels
    }

    pub fn load_nes_rom(&mut self, path: &std::path::Path) -> Result<(), NesParseError> {
        let file = std::fs::File::open(path)?;
        let catridge = Cartridge::from_nes(file)?;
        self.cpu.bus.attach_catridge(catridge);
        Ok(())
    }

    pub fn ppu(&self) -> &ppu::Ppu {
        &self.cpu.bus.ppu
    }

    pub fn cartridge(&self) -> Option<std::cell::Ref<Cartridge>> {
        Some(self.cpu.bus.cartridge.as_ref()?.borrow())
    }
}
