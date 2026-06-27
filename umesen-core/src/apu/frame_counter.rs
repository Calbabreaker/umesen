pub enum FrameCounterState {
    Quarter,
    QuarterHalf,
    None,
}

#[derive(Default)]
pub struct FrameCounter {
    cycles_counter: i32,
    five_step_mode: bool,
    irq_enabled: bool,
    pub irq_status: bool,
}

impl FrameCounter {
    /// Write to 0x4017
    pub fn write(&mut self, value: u8) {
        self.five_step_mode = value & 0b1000_0000 != 0;
        self.irq_enabled = value & 0b0100_0000 == 0;
        if !self.irq_enabled {
            self.irq_status = false;
        }
        // Delay counter reset by 3 or 4 cycles whether or not on a APU cycle
        self.cycles_counter = if self.cycles_counter % 2 == 0 { -3 } else { -4 }
    }

    /// Ran on every cpu cycle
    pub fn clock(&mut self) -> FrameCounterState {
        let count = self.cycles_counter;
        self.cycles_counter += 1;
        // Sequence from https://www.nesdev.org/wiki/APU_Frame_Counter
        // Note that this is in CPU cycles and includes one cycle delay
        if !self.five_step_mode && matches!(count, 0 | 29828 | 29829) {
            self.irq_status = self.irq_enabled
        }
        match (count, self.five_step_mode) {
            (7457, _) => FrameCounterState::Quarter,
            (14913, _) => FrameCounterState::QuarterHalf,
            (22371, _) => FrameCounterState::Quarter,
            (29829, false) | (37281, true) => {
                self.cycles_counter = 0;
                FrameCounterState::QuarterHalf
            }
            _ => FrameCounterState::None,
        }
    }
}
