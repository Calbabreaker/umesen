const DECAY_START: u8 = 15;

#[derive(Default)]
pub struct Envelope {
    timer_start: u8,
    timer: u8,
    constant_volume: bool,
    should_loop: bool,
    start: bool,
    decay_level: u8,
}

impl Envelope {
    /// Writes to 0x4000, 0x4004, 0x400c
    pub fn write(&mut self, value: u8) {
        self.timer_start = value & 0x0f;
        self.constant_volume = value & 0b0001_0000 != 0;
        self.should_loop = value & 0b0010_0000 != 0;
    }

    /// Clocked by every quarter frame frame counter
    pub fn clock(&mut self) {
        if self.start {
            self.decay_level = DECAY_START;
            self.timer = self.timer_start;
            self.start = false;
            return;
        }

        if self.timer == 0 {
            if self.decay_level != 0 {
                self.decay_level -= 1;
            } else if self.should_loop {
                self.decay_level = DECAY_START;
            }
            self.timer = self.timer_start;
        } else {
            self.timer -= 1;
        }
    }

    pub fn start(&mut self) {
        self.start = true;
    }

    pub fn volume(&self) -> f32 {
        if self.constant_volume {
            self.timer_start as f32
        } else {
            self.decay_level as f32
        }
    }
}
