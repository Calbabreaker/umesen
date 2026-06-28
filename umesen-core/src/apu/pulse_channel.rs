use super::{counters::LengthCounter, envelope::Envelope, sequencer::Sequencer, sweep::Sweep};

/// Waveform for the pulse wave for each duty cycle
/// https://www.nesdev.org/wiki/APU_Pulse#Sequencer_behavior
/// This will be multiplied by the envelope's decay level
const PULSE_WAVEFORM: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

/// Generator for pulse/square wave
pub struct PulseChannel {
    pub sequencer: Sequencer,
    pub envelope: Envelope,
    pub length_counter: LengthCounter,
    pub sweep: Sweep,
}

impl PulseChannel {
    pub fn new(sweep_ones_complement: bool) -> Self {
        Self {
            sequencer: Sequencer::new(&PULSE_WAVEFORM[0]),
            envelope: Envelope::default(),
            length_counter: LengthCounter::default(),
            sweep: Sweep::new(sweep_ones_complement),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        // Address for both pulse channels
        std::debug_assert_matches!(address, 0x4000..=0x4007);
        match address % 4 {
            0 => {
                self.sequencer.sequence = &PULSE_WAVEFORM[(value >> 6) as usize];
                self.envelope.write(value);
                self.length_counter.halt = value & 0b0010_0000 != 0;
            }
            1 => self.sweep.write(value),
            2 => self.sequencer.set_timer_low(value),
            3 => {
                self.envelope.start();
                self.sequencer.set_timer_high(value);
                self.length_counter.set_counter(value);
            }
            _ => unreachable!(),
        }
    }

    pub fn sample(&self) -> f32 {
        if self.length_counter.counter != 0 && !self.sweep.muted(&self.sequencer) {
            self.sequencer.sample() * self.envelope.volume()
        } else {
            0.
        }
    }
}
