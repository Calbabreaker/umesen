use ringbuf::traits::Producer;

use counters::{FrameCounter, FrameCounterState};
use noise_channel::NoiseChannel;
use pulse_channel::PulseChannel;
use triangle_channel::TriangleChannel;

mod counters;
mod envelope;
mod noise_channel;
mod pulse_channel;
mod sequencer;
mod sweep;
mod triangle_channel;

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Status: u8 {
        const PULSE_0 = 1;
        const PULSE_1 = 1 << 1;
        const TRIANGLE = 1 << 2;
        const NOISE = 1 << 3;
        const DMC = 1 << 4;
        const FRAME_IRQ = 1 << 6;
        const DMC_IRQ = 1 << 7;
    }
}

/// Emulated RP2A03 NTSC APU
pub struct Apu {
    pub(crate) sample_prod: Option<ringbuf::HeapProd<f32>>,
    pub(crate) sample_rate: f64,
    pub volume: f32,
    pulse_0: PulseChannel,
    pulse_1: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    frame_counter: FrameCounter,
    status: Status,
    cycles_since_sample: f64,
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            sample_prod: None,
            sample_rate: 0.,
            volume: 1.,
            pulse_0: PulseChannel::new(true),
            pulse_1: PulseChannel::new(false),
            triangle: TriangleChannel::default(),
            noise: NoiseChannel::default(),
            frame_counter: FrameCounter::default(),
            status: Status::empty(),
            cycles_since_sample: 0.,
        }
    }
}

impl Apu {
    pub fn write(&mut self, address: u16, value: u8) {
        std::debug_assert_matches!(address, 0x4000..=0x4017);
        match address {
            0x4000..=0x4003 => self.pulse_0.write(address, value),
            0x4004..=0x4007 => self.pulse_1.write(address, value),
            0x4008..=0x400b => self.triangle.write(address, value),
            0x400c..=0x400f => self.noise.write(address, value),
            0x4015 => self.set_status(value),
            0x4017 => {
                let state = self.frame_counter.write(value);
                self.handle_frame_state(state);
            }
            _ => (),
        }
    }

    pub fn read_status(&mut self) -> u8 {
        self.status
            .set(Status::FRAME_IRQ, self.frame_counter.irq.read_status());
        self.status.bits()
    }

    /// Ran on every CPU cycle
    pub fn clock(&mut self, cpu_cycles: u64) {
        self.triangle.sequencer.clock();
        if cpu_cycles.is_multiple_of(2) {
            self.pulse_0.sequencer.clock();
            self.pulse_1.sequencer.clock();
            self.noise.clock();
        }

        let state = self.frame_counter.clock();
        self.handle_frame_state(state);

        let sample = self.sample();
        if let Some(prod) = self.sample_prod.as_mut() {
            while self.cycles_since_sample > 0. {
                prod.try_push(sample).ok();
                self.cycles_since_sample -= crate::cpu::CLOCK_SPEED_HZ / self.sample_rate;
            }
        }

        self.cycles_since_sample += 1.;
    }

    pub fn irq_status(&self) -> bool {
        self.frame_counter.irq.status
    }

    pub fn reset(&mut self) {
        // Disable everything
        self.set_status(0);
    }

    fn set_status(&mut self, value: u8) {
        self.status = Status::from_bits_truncate(value);
        self.pulse_0
            .length_counter
            .set_enabled(self.status.contains(Status::PULSE_0));
        self.pulse_1
            .length_counter
            .set_enabled(self.status.contains(Status::PULSE_1));
        self.triangle
            .length_counter
            .set_enabled(self.status.contains(Status::TRIANGLE));
        self.noise
            .length_counter
            .set_enabled(self.status.contains(Status::NOISE));
    }

    fn handle_frame_state(&mut self, state: FrameCounterState) {
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

    fn sample(&self) -> f32 {
        let pulse_0 = self.pulse_0.sample();
        let pulse_1 = self.pulse_1.sample();
        let noise = self.noise.sample();
        let dmc = 0.;
        let triangle = self.triangle.sample();

        // Math from https://www.nesdev.org/wiki/APU_Mixer
        let pulse = pulse_0 + pulse_1;
        let tnd = triangle / 8227. + noise / 12241. + dmc / 22638.;
        let pulse_out = (95.88 * pulse) / (8128. + 100. * pulse);
        let tnd_out = (159.79 * tnd) / (1. + 100. * tnd);
        (tnd_out + pulse_out) * self.volume
    }
}
