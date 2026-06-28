use ringbuf::traits::{Producer, Split};

use channels::Channels;
use counters::FrameCounter;

mod channels;
mod counters;
mod envelope;
mod sequencer;
mod sweep;

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

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct ApuConfig {
    pub volume: f32,
    pub extra_filters: bool,
}

impl Default for ApuConfig {
    fn default() -> Self {
        Self {
            volume: 1.,
            extra_filters: false,
        }
    }
}

/// Emulated RP2A03 NTSC APU
#[derive(Default)]
pub struct Apu {
    pub config: ApuConfig,
    pub channels: Channels,
    pub(crate) speed_scale: f32,
    sample_sender: Option<SampleSender>,
    frame_counter: FrameCounter,
    status: Status,
}

impl Apu {
    pub fn write(&mut self, address: u16, value: u8) {
        std::debug_assert_matches!(address, 0x4000..=0x4017);
        match address {
            0x4000..=0x4013 => self.channels.write(address, value),
            0x4015 => {
                self.status = Status::from_bits_truncate(value);
                self.channels.set_enabled(self.status);
            }
            0x4017 => {
                let state = self.frame_counter.write(value);
                self.channels.handle_frame_state(state);
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
        self.channels.clock(cpu_cycles);

        let state = self.frame_counter.clock();
        self.channels.handle_frame_state(state);

        if let Some(sender) = self.sample_sender.as_mut() {
            sender.check_send(
                &self.channels,
                crate::cpu::CLOCK_SPEED_HZ / self.speed_scale,
                &self.config,
            );
        }
    }

    pub fn irq_status(&self) -> bool {
        self.frame_counter.irq.status
    }

    pub fn reset(&mut self) {
        self.status = Status::empty();
        self.channels.set_enabled(self.status);
    }

    /// Setup the audio buffer
    /// Returns the ring buffer consumer that contains the samples generated from the APU
    pub fn setup_audio_buffer(
        &mut self,
        sample_rate: u32,
        buffer_length: std::time::Duration,
    ) -> ringbuf::HeapCons<f32> {
        self.speed_scale = 1.;
        let sample_rate = sample_rate as f32;
        let size = sample_rate * buffer_length.as_secs_f32();
        let rb = ringbuf::SharedRb::new(size as usize);
        let (prod, cons) = rb.split();
        self.sample_sender = Some(SampleSender::new(sample_rate, prod));
        cons
    }
}

struct SampleSender {
    buffer_prod: ringbuf::HeapProd<f32>,
    sample_rate: f32,
    high_pass_filter: OnePoleFilter,
    extra_filters: [OnePoleFilter; 2],
    cycles_since_sample: f32,
}

impl SampleSender {
    pub fn new(sample_rate: f32, buffer_prod: ringbuf::HeapProd<f32>) -> Self {
        Self {
            buffer_prod,
            sample_rate,
            cycles_since_sample: 0.,
            // Filters specified from https://www.nesdev.org/wiki/APU_Mixer
            high_pass_filter: OnePoleFilter::new(90., sample_rate, false),
            extra_filters: [
                OnePoleFilter::new(440., sample_rate, false),
                OnePoleFilter::new(14000., sample_rate, true),
            ],
        }
    }

    fn check_send(&mut self, channels: &Channels, clock_speed: f32, config: &ApuConfig) {
        while self.cycles_since_sample > 0. {
            let mut sample = channels.sample() * config.volume;
            sample = self.high_pass_filter.process(sample);
            if config.extra_filters {
                for filter in self.extra_filters.iter_mut() {
                    sample = filter.process(sample);
                }
            }

            self.buffer_prod.try_push(sample).ok();
            self.cycles_since_sample -= clock_speed / self.sample_rate;
        }
        self.cycles_since_sample += 1.;
    }
}

struct OnePoleFilter {
    alpha: f32,
    prev_out: f32,
    prev_in: f32,
    low_pass: bool,
}

impl OnePoleFilter {
    fn new(cutoff_freq: f32, sample_rate: f32, low_pass: bool) -> Self {
        Self {
            alpha: (-std::f32::consts::TAU * cutoff_freq / sample_rate).exp(),
            prev_out: 0.,
            prev_in: 0.,
            low_pass,
        }
    }

    fn process(&mut self, sample: f32) -> f32 {
        // formulas from wikipedia
        let out = if self.low_pass {
            // y[i] := α * x[i] + (1-α) * y[i-1]
            self.alpha * sample + (1. - self.alpha) * self.prev_out
        } else {
            // y[i] := α × y[i−1] + α × (x[i] − x[i−1])
            self.alpha * self.prev_out + self.alpha * (sample - self.prev_in)
        };
        self.prev_out = out;
        self.prev_in = sample;
        out
    }
}
