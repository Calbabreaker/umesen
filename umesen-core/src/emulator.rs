use std::time::Instant;

use crate::{ppu, Cartridge, Cpu, CpuError, NesParseError};

/// High level struct for controlling the cpu
pub struct Emulator {
    pub cpu: Cpu,
    last_frame_time: Instant,
    frame_rate: f32,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            cpu: Cpu::default(),
            last_frame_time: Instant::now(),
            frame_rate: 0.,
        }
    }
}

impl Emulator {
    pub fn step(&mut self) -> Result<(), CpuError> {
        self.cpu.execute_next()
    }

    /// Keep stepping until a frame is generated
    pub fn next_frame(&mut self) {
        while !self.frame_complete() {
            if let Err(err) = self.cpu.execute_next() {
                log::warn!("Emulation error: {err}");
            }
        }
    }

    /// Keep clocking the cpu until it has caught up to the ellapsed seconds or there is a new frame
    /// Returns true if frame is complete
    pub fn clock_until_frame(&mut self, clocks_remaining: &mut u32) -> Result<bool, CpuError> {
        while *clocks_remaining > 0 {
            self.cpu.clock_until_execute(clocks_remaining)?;
            // Ensure only lastest frame is returned
            if *clocks_remaining < crate::cpu::CYCLES_PER_FRAME && self.frame_complete() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn load_nes_rom(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), NesParseError> {
        let file = std::fs::File::open(path)?;
        let catridge = Cartridge::from_nes(file)?;
        self.cpu.bus.attach_catridge(catridge);
        Ok(())
    }

    fn frame_complete(&mut self) -> bool {
        let ppu = &mut self.cpu.bus.ppu;
        if ppu.frame_complete {
            ppu.frame_complete = false;
            self.frame_rate = 1. / self.last_frame_time.elapsed().as_secs_f32();
            self.last_frame_time = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn frame_rate(&self) -> f32 {
        self.frame_rate
    }

    pub fn ppu(&self) -> &ppu::Ppu {
        &self.cpu.bus.ppu
    }

    pub fn cartridge(&self) -> Option<std::cell::Ref<Cartridge>> {
        Some(self.cpu.bus.cartridge.as_ref()?.borrow())
    }

    pub fn debug_log(&self) -> String {
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
            self.cpu.bus.cpu_cycles_total,
        )
    }
}
