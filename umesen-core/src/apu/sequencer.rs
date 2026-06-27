pub struct Sequencer {
    /// 11 bit for the sequencer to go to the next step
    timer: u16,
    pub timer_start: u16,
    step: usize,
    pub sequence: &'static [u8],
}

impl Sequencer {
    pub fn new(sequence: &'static [u8]) -> Self {
        Self {
            sequence,
            timer_start: 0,
            step: 0,
            timer: 0,
        }
    }

    pub fn sample(&self) -> f32 {
        self.sequence[self.step] as f32
    }

    pub fn clock(&mut self) {
        if self.timer == 0 {
            self.step = (self.step + 1) % self.sequence.len();
            self.timer = self.timer_start;
        } else {
            self.timer -= 1;
        }
    }

    pub fn set_timer_low(&mut self, value: u8) {
        self.timer_start = (value as u16) | self.timer_start & (0xff00);
    }

    pub fn set_timer_high(&mut self, value: u8) {
        self.timer_start = ((value as u16 & 0b0000_0111) << 8) | self.timer_start & (0x00ff);
        self.step = 0;
    }
}
