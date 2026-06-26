use ringbuf::traits::Split;

use crate::{
    Cartridge, Controller, Cpu, Ppu,
    cartridge::NesParseError,
    cpu::{CLOCK_SPEED_HZ, CYCLES_PER_FRAME, CpuError},
    ppu::ScreenPixels,
};

/// High level struct for controlling the cpu
pub struct Emulator {
    pub cpu: Cpu,
    pub speed: f64,
    pub running: bool,
    last_frame_time: std::time::Instant,
    frame_rate: f64,
    clocks_remaining: f64,
    last_update_time: std::time::Instant,
    audio_sample_rate: f64,
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            last_update_time: std::time::Instant::now(),
            audio_sample_rate: 0.,
            running: true,
            cpu: Cpu::default(),
            last_frame_time: std::time::Instant::now(),
            frame_rate: 0.,
            clocks_remaining: 0.,
            speed: 1.,
        }
    }
}

impl Emulator {
    pub fn setup_audio_buffer(
        &mut self,
        sample_rate: u32,
        buffer_length: std::time::Duration,
    ) -> ringbuf::HeapCons<f32> {
        self.audio_sample_rate = sample_rate as f64;
        let size = self.audio_sample_rate * buffer_length.as_secs_f64();
        let rb = ringbuf::SharedRb::new(size as usize);
        let (prod, cons) = rb.split();
        self.cpu.bus.apu.sample_prod = Some(prod);
        cons
    }

    /// Keep stepping until a frame is generated
    pub fn next_frame(&mut self) -> Result<(), CpuError> {
        while !self.ppu().frame_complete() {
            self.cpu.execute_next()?;
        }
        Ok(())
    }

    /// Calculates the delta time that has passed since calling this function and clock the cpu
    /// required for that amount of time
    pub fn update(
        &mut self,
        mut on_frame_completed: impl FnMut(&ScreenPixels),
    ) -> Result<(), CpuError> {
        let delta = self.last_update_time.elapsed().as_secs_f64() * self.speed;
        self.cpu.bus.apu.sample_rate = self.audio_sample_rate / self.speed;
        self.last_update_time = std::time::Instant::now();
        if !self.running {
            self.clocks_remaining = 0.;
            return Ok(());
        }

        self.clocks_remaining += delta * CLOCK_SPEED_HZ;
        while self.clocks_remaining > 0. {
            self.clocks_remaining -= self.cpu.execute_next()? as f64;
            if self.ppu().frame_complete() && self.clocks_remaining < CYCLES_PER_FRAME {
                self.frame_rate = 1. / self.last_frame_time.elapsed().as_secs_f64();
                self.last_frame_time = std::time::Instant::now();
                on_frame_completed(&self.ppu().screen_pixels);
            }
        }
        Ok(())
    }

    pub fn load_nes_rom(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), NesParseError> {
        self.last_update_time = std::time::Instant::now();
        let file = std::fs::File::open(path)?;
        let catridge = Cartridge::from_nes(file)?;
        self.cpu.bus.attach_catridge(catridge);
        self.cpu.reset();
        Ok(())
    }

    pub fn frame_rate(&self) -> f64 {
        self.frame_rate
    }

    pub fn ppu(&mut self) -> &mut Ppu {
        &mut self.cpu.bus.ppu
    }

    pub fn cartridge(&self) -> Option<std::cell::Ref<'_, Cartridge>> {
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
            self.cpu.bus.ppu.registers.scanline,
            self.cpu.bus.ppu.registers.dot,
            self.cpu.bus.cpu_cycles_total,
        )
    }

    pub fn controller(&mut self, number: u8) -> &mut Controller {
        &mut self.cpu.bus.controllers[number as usize]
    }
}
