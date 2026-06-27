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
#[derive(Default)]
pub struct PulseChannel {
    /// 11 bit for the sequencer to go to the next step
    period_timer: u16,
    period: u16,
    sequencer_step: usize,
    duty_cycle: u8,
}

impl PulseChannel {
    pub fn write_u8(&mut self, address: u16, value: u8) {
        // Address for both pulse channels
        std::debug_assert_matches!(address, 0x4000..=0x4007);
        match address % 4 {
            0 => self.duty_cycle = value >> 6,
            1 => (),
            // Period timer start low bits
            2 => self.period = (value as u16) | self.period & (0xff00),
            // Period timer start high 3 bits
            3 => self.period = ((value as u16 & 0b0000_0111) << 8) | self.period & (0x00ff),
            _ => unreachable!(),
        }
    }

    /// Ran on every apu cycle
    pub fn clock(&mut self) {
        if self.period_timer == 0 {
            self.sequencer_step = (self.sequencer_step + 1) % PULSE_WAVEFORM[0].len();
            self.period_timer = self.period;
        } else {
            self.period_timer -= 1;
        }
    }

    pub fn sample(&self) -> f32 {
        if self.period >= 8 {
            PULSE_WAVEFORM[self.duty_cycle as usize][self.sequencer_step] as f32 * 15.
        } else {
            0.
        }
    }
}
