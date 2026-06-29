use crate::apu::ApuConfig;

use super::{Status, counters::FrameCounterState};
use noise::NoiseChannel;
use pulse::PulseChannel;
use triangle::TriangleChannel;

mod noise;
mod pulse;
mod triangle;

pub struct Channels {
    pulse_0: PulseChannel,
    pulse_1: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
}

impl Default for Channels {
    fn default() -> Self {
        Self {
            pulse_0: PulseChannel::new(true),
            pulse_1: PulseChannel::new(false),
            triangle: TriangleChannel::default(),
            noise: NoiseChannel::default(),
        }
    }
}

impl Channels {
    pub fn write(&mut self, address: u16, value: u8) {
        std::debug_assert_matches!(address, 0x4000..=0x4013);
        match address {
            0x4000..=0x4003 => self.pulse_0.write(address, value),
            0x4004..=0x4007 => self.pulse_1.write(address, value),
            0x4008..=0x400b => self.triangle.write(address, value),
            0x400c..=0x400f => self.noise.write(address, value),
            0x4010..=0x4013 => (),
            _ => unreachable!(),
        }
    }

    pub fn clock(&mut self, cpu_cycles: u64) {
        self.triangle.clock();
        if cpu_cycles.is_multiple_of(2) {
            self.pulse_0.sequencer.clock();
            self.pulse_1.sequencer.clock();
            self.noise.clock();
        }
    }

    pub fn handle_frame_state(&mut self, state: FrameCounterState) {
        if state == FrameCounterState::Half {
            // Half frame clocks
            self.pulse_0.length_counter.clock();
            self.pulse_0.sweep.clock(&mut self.pulse_0.sequencer);
            self.pulse_1.length_counter.clock();
            self.pulse_1.sweep.clock(&mut self.pulse_1.sequencer);
            self.triangle.length_counter.clock();
            self.noise.length_counter.clock();
        }

        if matches!(state, FrameCounterState::Half | FrameCounterState::Quarter) {
            // Quarter frame clocks
            self.pulse_0.envelope.clock();
            self.pulse_1.envelope.clock();
            self.noise.envelope.clock();
            self.triangle.clock_linear_counter();
        }
    }

    pub fn set_enabled(&mut self, status: Status) {
        self.pulse_0
            .length_counter
            .set_enabled(status.contains(Status::PULSE_0));
        self.pulse_1
            .length_counter
            .set_enabled(status.contains(Status::PULSE_1));
        self.triangle
            .length_counter
            .set_enabled(status.contains(Status::TRIANGLE));
        self.noise
            .length_counter
            .set_enabled(status.contains(Status::NOISE));
    }

    pub fn sample(&self, config: &ApuConfig) -> f32 {
        let pulse_0 = self.pulse_0.sample() as f32 * config.pulse_0_volume;
        let pulse_1 = self.pulse_1.sample() as f32 * config.pulse_1_volume;
        let noise = self.noise.sample() as f32 * config.noise_volume;
        let triangle = self.triangle.sample() as f32 * config.triangle_volume;
        let dmc = 0.;

        // Math from https://www.nesdev.org/wiki/APU_Mixer
        let pulse = pulse_0 + pulse_1;
        let tnd = triangle / 8227. + noise / 12241. + dmc / 22638.;
        let pulse_out = (95.88 * pulse) / (8128. + 100. * pulse);
        let tnd_out = (159.79 * tnd) / (1. + 100. * tnd);
        (tnd_out + pulse_out) * config.volume
    }
}
