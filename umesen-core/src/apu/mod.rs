use ringbuf::traits::Producer;

use counters::{FrameCounter, FrameCounterState};
use pulse_channel::PulseChannel;

mod counters;
mod envelope;
mod pulse_channel;
mod sequencer;

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
#[derive(Default)]
pub struct Apu {
    pub(crate) sample_prod: Option<ringbuf::HeapProd<f32>>,
    pub(crate) sample_rate: f64,
    pub volume: f32,
    pulse_0: PulseChannel,
    pulse_1: PulseChannel,
    frame_counter: FrameCounter,
    cycles_since_sample: f64,
    status: Status,
}

impl Apu {
    pub fn write(&mut self, address: u16, value: u8) {
        std::debug_assert_matches!(address, 0x4000..=0x4017);
        match address {
            0x4000..=0x4003 => self.pulse_0.write(address, value),
            0x4004..=0x4007 => self.pulse_1.write(address, value),
            0x4015 => {
                self.status = Status::from_bits_truncate(value);
                self.pulse_0
                    .length_counter
                    .set_enabled(self.status.contains(Status::PULSE_0));
                self.pulse_1
                    .length_counter
                    .set_enabled(self.status.contains(Status::PULSE_1));
            }
            0x4017 => self.frame_counter.write(value),
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
        if cpu_cycles.is_multiple_of(2) {
            self.pulse_0.sequencer.clock();
            self.pulse_1.sequencer.clock();
        }

        let state = self.frame_counter.clock();
        if matches!(state, FrameCounterState::Half) {
            self.pulse_0.length_counter.clock();
            self.pulse_1.length_counter.clock();

            if matches!(state, FrameCounterState::Quarter) {
                self.pulse_0.envelope.clock();
                self.pulse_1.envelope.clock();
            }
        }

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
        self.write(0x4015, 0);
    }

    fn sample(&self) -> f32 {
        // Math from https://www.nesdev.org/wiki/APU_Mixer
        let pulse_out = 95.88 / (8128. / (self.pulse_0.sample() + self.pulse_1.sample()) + 100.);

        pulse_out * self.volume
    }
}
