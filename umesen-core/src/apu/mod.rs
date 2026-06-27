use ringbuf::traits::Producer;

use crate::cpu::CLOCK_SPEED_HZ;
use pulse_channel::PulseChannel;

mod pulse_channel;

/// Emulated RP2A03 NTSC APU
#[derive(Default)]
pub struct Apu {
    pub sample_prod: Option<ringbuf::HeapProd<f32>>,
    pub sample_rate: f64,
    pub pulse_0: PulseChannel,
    pub pulse_1: PulseChannel,
    cycles_since_sample: f64,
}

impl Apu {
    pub fn write_u8(&mut self, address: u16, value: u8) {
        std::debug_assert_matches!(address, 0x4000..=0x4017);
        match address {
            0x4000..=0x4003 => self.pulse_0.write_u8(address, value),
            0x4004..=0x4007 => self.pulse_1.write_u8(address, value),
            _ => (),
        }
    }

    /// Ran on every CPU cycle
    pub fn clock(&mut self, cpu_cycles: u64) {
        if cpu_cycles.is_multiple_of(2) {
            self.pulse_0.clock();
            self.pulse_1.clock();
        }

        let sample = self.sample();
        if let Some(prod) = self.sample_prod.as_mut() {
            while self.cycles_since_sample > 0. {
                prod.try_push(sample).ok();
                self.cycles_since_sample -= CLOCK_SPEED_HZ / self.sample_rate;
            }
        }

        self.cycles_since_sample += 1.;
    }

    fn sample(&self) -> f32 {
        // Math from https://www.nesdev.org/wiki/APU_Mixer
        let pulse_out = 95.88 / (8128. / (self.pulse_0.sample() + self.pulse_1.sample()) + 100.);

        pulse_out
    }
}
