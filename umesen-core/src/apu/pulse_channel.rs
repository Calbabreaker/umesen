use super::{envelope::Envelope, sequencer::Sequencer};

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
}

impl Default for PulseChannel {
    fn default() -> Self {
        Self {
            sequencer: Sequencer::new(&PULSE_WAVEFORM[0]),
            envelope: Envelope::default(),
        }
    }
}

impl PulseChannel {
    pub fn write(&mut self, address: u16, value: u8) {
        // Address for both pulse channels
        std::debug_assert_matches!(address, 0x4000..=0x4007);
        match address % 4 {
            0 => {
                self.sequencer.sequence = &PULSE_WAVEFORM[(value >> 6) as usize];
                self.envelope.write(value);
            }
            1 => (),
            2 => self.sequencer.set_timer_low(value),
            3 => {
                self.envelope.start();
                self.sequencer.set_timer_high(value);
            }
            _ => unreachable!(),
        }
    }

    pub fn sample(&self) -> f32 {
        if self.sequencer.timer_start >= 8 {
            self.sequencer.sample() * self.envelope.volume()
        } else {
            0.
        }
    }
}
