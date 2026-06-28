use super::{counters::TimerCounter, sequencer::Sequencer};

#[derive(Default)]
pub struct Sweep {
    enabled: bool,
    timer: TimerCounter<u8>,
    negate: bool,
    shift_count: u8,
    ones_complement: bool,
}

impl Sweep {
    pub fn new(ones_complement: bool) -> Self {
        Self {
            ones_complement,
            ..Default::default()
        }
    }

    pub fn write(&mut self, value: u8) {
        self.enabled = value & 0b1000_0000 != 0;
        self.timer.start = (value & 0b0111_0000) >> 4;
        self.timer.counter = self.timer.start + 1;
        self.negate = value & 0b0000_1000 != 0;
        self.shift_count = value & 0b0000_0111;
    }

    /// Clocked on every half frame
    /// Calculates target period of period
    pub fn clock(&mut self, sequencer: &mut Sequencer) {
        if self.timer.clock() && self.enabled && self.shift_count != 0 && !self.muted(sequencer) {
            sequencer.timer.start = self.target_period(sequencer);
        }
    }

    pub fn muted(&self, sequencer: &Sequencer) -> bool {
        sequencer.timer.start < 8 || self.target_period(sequencer) > 0x7ff
    }

    pub fn target_period(&self, sequencer: &Sequencer) -> u16 {
        let period = sequencer.timer.start;
        let change = period >> self.shift_count;
        // Rotate right an 11 bit number
        if self.negate {
            period
                .saturating_sub(change)
                .saturating_sub(self.ones_complement as u16)
        } else {
            period + change
        }
    }
}
