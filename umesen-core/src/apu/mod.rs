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
    pub triangle_volume: f32,
    pub pulse_0_volume: f32,
    pub pulse_1_volume: f32,
    pub noise_volume: f32,
}

impl Default for ApuConfig {
    fn default() -> Self {
        Self {
            volume: 1.,
            triangle_volume: 1.,
            pulse_0_volume: 1.,
            pulse_1_volume: 1.,
            noise_volume: 1.,
        }
    }
}

/// Emulated RP2A03 NTSC APU
#[derive(Default)]
pub struct Apu {
    pub config: ApuConfig,
    pub(crate) speed_scale: f32,
    channels: Channels,
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
            sender.check_send(|| self.channels.sample(&self.config), self.speed_scale);
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
    high_pass: OnePoleFilter<true>,
    cycles_since_sample: f32,
}

impl SampleSender {
    pub fn new(sample_rate: f32, buffer_prod: ringbuf::HeapProd<f32>) -> Self {
        Self {
            buffer_prod,
            sample_rate,
            cycles_since_sample: 0.,
            high_pass: OnePoleFilter::default(),
        }
    }

    fn check_send(&mut self, get_sample: impl Fn() -> f32, speed_scale: f32) {
        while self.cycles_since_sample > 0. {
            let real_sample_rate = self.sample_rate / speed_scale;
            // Include low freq high pass filter to get rid of DC bias
            let sample = self.high_pass.process(get_sample(), real_sample_rate, 20.);

            self.buffer_prod.try_push(sample).ok();
            self.cycles_since_sample -= crate::cpu::CLOCK_SPEED_HZ / real_sample_rate;
        }
        self.cycles_since_sample += 1.;
    }
}

#[derive(Default)]
struct OnePoleFilter<const HIGH_PASS: bool> {
    prev_out: f32,
    prev_in: f32,
}

impl<const HIGH_PASS: bool> OnePoleFilter<{ HIGH_PASS }> {
    fn process(&mut self, sample: f32, sample_rate: f32, cutoff_freq: f32) -> f32 {
        let rc = 1. / (std::f32::consts::TAU * cutoff_freq);
        let dt = 1. / sample_rate;
        // formulas from wikipedia
        let out = if HIGH_PASS {
            // y[i] := α × y[i−1] + α × (x[i] − x[i−1])
            let alpha = rc / (rc + dt);
            alpha * self.prev_out + alpha * (sample - self.prev_in)
        } else {
            // y[i] := α * x[i] + (1-α) * y[i-1]
            let alpha = dt / (rc + dt);
            alpha * sample + (1. - alpha) * self.prev_out
        };
        self.prev_out = out;
        self.prev_in = sample;
        out
    }
}
