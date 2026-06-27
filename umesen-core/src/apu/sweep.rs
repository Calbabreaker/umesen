use super::sequencer::Sequencer;

#[derive(Default)]
pub struct Sweep {
    enabled: bool,
    timer: u8,
    timer_start: u8,
    negate: bool,
    shift_count: u8,
    pub ones_complement: bool,
}

impl Sweep {
    pub fn write(&mut self, value: u8) {
        self.enabled = value & 0b1000_0000 != 0;
        self.timer_start = (value & 0b0111_0000) >> 4;
        self.timer = self.timer_start + 1;
        self.negate = value & 0b0000_1000 != 0;
        self.shift_count = value & 0b0000_0111;
    }

    /// Clocked on every half frame
    /// Calculates target period of period
    pub fn clock(&mut self, sequencer: &mut Sequencer) {
        if self.timer == 0 {
            if self.enabled && self.shift_count != 0 && !self.muted(sequencer) {
                // println!(
                //     "{} {} {}",
                //     self.target_period(sequencer),
                //     self.shift_count,
                //     sequencer.timer_start
                // );
                sequencer.timer_start = self.target_period(sequencer);
            }
            self.timer = self.timer_start;
        } else {
            self.timer -= 1;
        }
    }

    pub fn muted(&self, sequencer: &Sequencer) -> bool {
        sequencer.timer_start < 8 || self.target_period(sequencer) > 0x7ff
    }

    pub fn target_period(&self, sequencer: &Sequencer) -> u16 {
        let change = sequencer.timer_start >> self.shift_count;
        // Rotate right an 11 bit number
        if self.negate {
            sequencer
                .timer_start
                .saturating_sub(change)
                .saturating_sub(self.ones_complement as u16)
        } else {
            sequencer.timer_start + change
        }
    }
}
