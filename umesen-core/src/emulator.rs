use crate::{Cartridge, Cpu, NesParseError};

/// High level struct for controlling the cpu
#[derive(Default)]
pub struct Emulator {
    pub cpu: Cpu,
}

impl Emulator {
    pub fn next_frame(&mut self) {
        todo!()
    }

    pub fn step(&mut self) {
        if let Err(err) = self.cpu.execute_next() {
            log::error!("{err}")
        }
    }

    pub fn load_nes_rom(&mut self, path: &std::path::Path) -> Result<(), NesParseError> {
        let file = std::fs::File::open(path)?;
        let catridge = Cartridge::from_nes(file)?;
        self.cpu.bus.attach_catridge(catridge);
        Ok(())
    }
}
