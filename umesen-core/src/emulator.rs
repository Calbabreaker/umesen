use std::time::{Duration, Instant};

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
    pub fn step(&mut self) {
        loop {
            match self.cpu.clock() {
                Err(err) => log::error!("{err}"),
                Ok(true) => break,
                Ok(false) => (),
            }
        }
    }

    /// Keep stepping until a frame is generated
    pub fn next_frame(&mut self) {
        while !self.cpu.bus.ppu.frame_complete {
            self.step();
        }
        self.cpu.bus.ppu.frame_complete = false;
    }

    /// Keep clocking the cpu until it has caught up to the current time
    pub fn clock_until_caught_up(&mut self, elapsed_secs: impl Into<f64>) -> Result<(), CpuError> {
        let cycles_to_clock = (elapsed_secs.into() * crate::cpu::CLOCK_SPEED_HZ as f64).round();
        for _ in 0..cycles_to_clock as usize {
            self.cpu.clock()?;
        }
        Ok(())
    }

    pub fn load_nes_rom(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), NesParseError> {
        let file = std::fs::File::open(path)?;
        let catridge = Cartridge::from_nes(file)?;
        self.cpu.bus.attach_catridge(catridge);
        Ok(())
    }

    pub fn frame_complete(&mut self) -> bool {
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

    pub fn get_debug_log(&self) -> String {
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
            self.cpu.cpu_cycles_total,
        )
    }
}
