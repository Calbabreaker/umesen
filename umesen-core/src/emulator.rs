use crate::{ppu, Cartridge, Cpu, CpuError, NesParseError};

/// High level struct for controlling the cpu
#[derive(Default)]
pub struct Emulator {
    pub cpu: Cpu,
}

impl Emulator {
    pub fn step(&mut self) {
        if let Err(err) = self.cpu.execute_next() {
            log::error!("{err}")
        }
    }

    pub fn next_frame(&mut self) {
        while !self.cpu.bus.ppu.frame_complete {
            self.step();
        }
        self.cpu.bus.ppu.frame_complete = false;
    }

    pub fn next_frame_debug(&mut self) -> Result<(), CpuError> {
        while !self.cpu.bus.ppu.frame_complete {
            self.cpu.execute_next()?;
        }
        self.cpu.bus.ppu.frame_complete = false;
        Ok(())
    }

    pub fn load_nes_rom(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), NesParseError> {
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

    pub fn get_debug_state(&self) -> String {
        format!(
            "{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{: >3},{: >3} CYC:{}",
            self.cpu.pc,
            self.cpu.a,
            self.cpu.x,
            self.cpu.y,
            self.cpu.flags.bits(),
            self.cpu.sp,
            self.ppu().scanline,
            self.ppu().dot,
            self.cpu.bus.cpu_cycles,
        )
    }
}
